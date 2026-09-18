use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        Html,
    },
    routing::{get, post},
    Json, Router,
};
use colored::*;
use futures_util::stream::Stream;
use tokio_stream::StreamExt;
use paraclea_core::{
    bible::{self, BibleReader},
    config::Config,
    dendrite::{Dendrite, DendriteStore},
    library::LibraryEngine,
    mesh::ReticulumEngine,
    ollama::{ChatMessage, OllamaClient},
    persona::PersonaManager,
    qdrant::QdrantClient,
};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

const HTML_CONTENT: &str = include_str!("../public/index.html");

#[derive(Clone)]
#[allow(dead_code)]
struct AppState {
    ollama: Arc<OllamaClient>,
    persona: Arc<PersonaManager>,
    library: Arc<tokio::sync::RwLock<LibraryEngine>>,
    dendrite_store: Option<Arc<DendriteStore>>,
    dendrite_graph: Arc<Dendrite>,
    bible_reader: Arc<tokio::sync::RwLock<Option<BibleReader>>>,
    mesh: Option<Arc<ReticulumEngine>>,
    qdrant: Arc<QdrantClient>,
    config: Arc<Config>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("{}", "╔══════════════════════════════════════════════════════════════╗".purple().bold());
    println!("{}", "║     PARACLEA AI ASSISTANT — DESKTOP APPLICATION SERVER       ║".yellow().bold());
    println!("{}", "╚══════════════════════════════════════════════════════════════╝".purple().bold());

    let config_path = Config::find_or_default_config_path();
    let cfg = Config::load(&config_path).unwrap_or_default();
    let config = Arc::new(cfg);

    let home = paraclea_core::home_dir();
    let persona_dir = home.join(".paraclea/persona");
    let persona = Arc::new(PersonaManager::new(persona_dir).unwrap_or_else(|_| PersonaManager { persona_dir: PathBuf::from("persona") }));

    // Gracefully initialize Ollama client
    let ollama_client = OllamaClient::new(&config.model.ollama.url, &config.model.ollama.model)
        .or_else(|_| OllamaClient::new("http://localhost:11434", "ministral-3:3b"))
        .expect("Ollama client initialization");
    let ollama = Arc::new(ollama_client);

    // Gracefully initialize Qdrant client
    let qdrant_client = QdrantClient::new(&config.vector_db.qdrant_url)
        .or_else(|_| QdrantClient::new("http://localhost:6333"))
        .expect("Qdrant client initialization");
    let qdrant = Arc::new(qdrant_client);

    let library = Arc::new(tokio::sync::RwLock::new(LibraryEngine::load_auto()));
    let mesh = ReticulumEngine::new().ok().map(Arc::new);

    let dendrite_graph = Arc::new(Dendrite::new());
    let db_path = home.join(".paraclea/dendrite.db");
    let dendrite_store = DendriteStore::open(&db_path).ok().map(Arc::new);
    if let Some(ref store) = dendrite_store {
        let _ = store.load_all(&dendrite_graph);
    }

    let initial_reader = BibleReader::load_auto().ok();
    let bible_reader = Arc::new(tokio::sync::RwLock::new(initial_reader));

    let state = AppState {
        ollama,
        persona,
        library,
        dendrite_store,
        dendrite_graph,
        bible_reader,
        mesh,
        qdrant,
        config,
    };

    let app = Router::new()
        .route("/", get(serve_html))
        .route("/api/languages", get(get_languages))
        .route("/api/translations", get(get_translations))
        .route("/api/bible/books", get(get_bible_books))
        .route("/api/bible/read", get(read_bible_chapter))
        .route("/api/library/books", get(get_library_books))
        .route("/api/library/read", get(read_library_chapter))
        .route("/api/chat", post(handle_chat))
        .route("/api/chat/stream", post(handle_chat_stream))
        .route("/api/memory", get(get_memory_nodes))
        .route("/api/mesh", get(get_mesh_status))
        .route("/api/mesh/mailbox", get(get_mesh_mailbox))
        .route("/api/mesh/send", post(send_mesh_message))
        .route("/api/matrix", get(get_matrix_results))
        .route("/api/doctor", get(run_doctor_checks))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(7860);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("{}", format!("  ✓ Server listening on http://{}", addr).green().bold());
    println!("{}", "  ✓ Launching Desktop Web Browser...".yellow().bold());

    tokio::spawn(async move {
        let _ = open::that(format!("http://{}", addr));
    });

    axum::serve(listener, app).await?;

    Ok(())
}

async fn serve_html() -> Html<&'static str> {
    Html(HTML_CONTENT)
}

#[derive(Serialize)]
struct LanguageDto {
    code: String,
    name: String,
}

async fn get_languages() -> Json<Vec<LanguageDto>> {
    let raw = BibleReader::list_languages();
    let res = raw.into_iter().map(|l| LanguageDto { code: l.code, name: l.name }).collect();
    Json(res)
}

#[derive(Deserialize)]
struct TransParams {
    lang: String,
}

#[derive(Serialize)]
struct TransDto {
    tag: String,
    name: String,
}

async fn get_translations(Query(params): Query<TransParams>) -> Json<Vec<TransDto>> {
    let raw = BibleReader::list_translations_for_lang(&params.lang);
    let res = raw.into_iter().map(|t| TransDto { tag: t.tag, name: t.name }).collect();
    Json(res)
}

#[derive(Serialize)]
struct BibleBookDto {
    name: String,
    testament: String,
    total_chapters: usize,
}

async fn get_bible_books(State(state): State<AppState>) -> Json<Vec<BibleBookDto>> {
    let reader_lock = state.bible_reader.read().await;
    let mut books = Vec::new();

    for &b in bible::OLD_TESTAMENT_BOOKS {
        let total_chapters = reader_lock.as_ref().and_then(|r| r.get_chapter_count(b)).unwrap_or(1);
        books.push(BibleBookDto {
            name: b.to_string(),
            testament: "Old Testament".to_string(),
            total_chapters,
        });
    }

    for &b in bible::NEW_TESTAMENT_BOOKS {
        let total_chapters = reader_lock.as_ref().and_then(|r| r.get_chapter_count(b)).unwrap_or(1);
        books.push(BibleBookDto {
            name: b.to_string(),
            testament: "New Testament".to_string(),
            total_chapters,
        });
    }

    Json(books)
}

#[derive(Deserialize)]
struct ReadBibleParams {
    tag: String,
    book: String,
    chapter: usize,
}

#[derive(Serialize)]
struct BibleVerseDto {
    verse: usize,
    text: String,
}

#[derive(Serialize)]
struct BibleChapterResponse {
    tag: String,
    book: String,
    chapter: usize,
    total_chapters: usize,
    verses: Vec<BibleVerseDto>,
}

async fn read_bible_chapter(
    State(state): State<AppState>,
    Query(params): Query<ReadBibleParams>,
) -> Json<BibleChapterResponse> {
    let reader_guard = state.bible_reader.read().await;
    let reader = if let Some(path) = bible::find_json_bible_file(&params.tag) {
        BibleReader::load_primary(&path).ok()
    } else {
        reader_guard.clone()
    };

    if let Some(r) = reader {
        let total_chapters = r.get_chapter_count(&params.book).unwrap_or(1);
        let verses = r
            .read_chapter(&params.book, params.chapter)
            .unwrap_or_default()
            .into_iter()
            .map(|(v, text)| BibleVerseDto { verse: v, text })
            .collect();

        return Json(BibleChapterResponse {
            tag: params.tag,
            book: params.book,
            chapter: params.chapter,
            total_chapters,
            verses,
        });
    }

    Json(BibleChapterResponse {
        tag: params.tag,
        book: params.book,
        chapter: params.chapter,
        total_chapters: 0,
        verses: Vec::new(),
    })
}

#[derive(Serialize)]
struct ChapterInfoDto {
    chapter_number: usize,
    title: String,
}

#[derive(Serialize)]
struct BookSummaryDto {
    title: String,
    category: String,
    author: String,
    chapters_count: usize,
    chapters: Vec<ChapterInfoDto>,
}

async fn get_library_books(State(state): State<AppState>) -> Json<Vec<BookSummaryDto>> {
    let lib = state.library.read().await;
    let mut list = Vec::new();

    for book in &lib.books {
        let chapters = book
            .chapters
            .iter()
            .map(|c| ChapterInfoDto {
                chapter_number: c.chapter_number,
                title: c.title.clone(),
            })
            .collect();

        list.push(BookSummaryDto {
            title: book.title.clone(),
            category: book.category.clone(),
            author: book.author.clone().unwrap_or_else(|| "Unknown".to_string()),
            chapters_count: book.chapters.len(),
            chapters,
        });
    }

    Json(list)
}

#[derive(Deserialize)]
struct ReadLibParams {
    title: String,
    chapter: usize,
}

#[derive(Serialize)]
struct ChapterContentResponse {
    book_title: String,
    chapter_number: usize,
    total_chapters: usize,
    chapter_title: String,
    content: String,
}

async fn read_library_chapter(
    State(state): State<AppState>,
    Query(params): Query<ReadLibParams>,
) -> Json<ChapterContentResponse> {
    let lib = state.library.read().await;
    let p = params.title.trim().to_lowercase();

    let target_book = lib.books.iter().find(|b| {
        let t = b.title.to_lowercase();
        t == p || t.contains(&p) || p.contains(&t)
    });

    if let Some(book) = target_book {
        let total_chapters = book.chapters.len();
        let ch_idx = if params.chapter >= 1 && params.chapter <= total_chapters {
            params.chapter - 1
        } else {
            0
        };

        if let Some(ch) = book.chapters.get(ch_idx) {
            return Json(ChapterContentResponse {
                book_title: book.title.clone(),
                chapter_number: ch.chapter_number,
                total_chapters,
                chapter_title: ch.title.clone(),
                content: ch.content.clone(),
            });
        }
    }

    Json(ChapterContentResponse {
        book_title: params.title,
        chapter_number: 1,
        total_chapters: 0,
        chapter_title: "Chapter Not Found".to_string(),
        content: "Content not available for this chapter.".to_string(),
    })
}

#[derive(Deserialize)]
struct ChatRequest {
    message: String,
}

#[derive(Serialize)]
struct ChatResponse {
    reply: String,
}

async fn handle_chat(
    State(state): State<AppState>,
    Json(payload): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (StatusCode, Json<ChatResponse>)> {
    let system_prompt = state.persona.build_system_prompt();
    let msgs = vec![
        ChatMessage { role: "system".to_string(), content: system_prompt },
        ChatMessage { role: "user".to_string(), content: payload.message },
    ];

    match state.ollama.chat(msgs).await {
        Ok(reply) => Ok(Json(ChatResponse { reply })),
        Err(err) => Err((
            StatusCode::BAD_GATEWAY,
            Json(ChatResponse {
                reply: format!("⚠️ Unable to connect to Ollama model: {}", err),
            }),
        )),
    }
}

async fn handle_chat_stream(
    State(state): State<AppState>,
    Json(payload): Json<ChatRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    let system_prompt = state.persona.build_system_prompt();
    let msgs = vec![
        ChatMessage { role: "system".to_string(), content: system_prompt },
        ChatMessage { role: "user".to_string(), content: payload.message },
    ];

    let ollama = state.ollama.clone();
    tokio::spawn(async move {
        let _ = ollama
            .chat_stream(msgs, move |token| {
                let _ = tx.send(token.to_string());
            })
            .await;
    });

    let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx).map(|token| {
        Ok(Event::default().data(token))
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

#[derive(Serialize)]
struct MemoryNodeDto {
    id: String,
    label: String,
    content: String,
    node_type: String,
}

async fn get_memory_nodes(State(state): State<AppState>) -> Json<Vec<MemoryNodeDto>> {
    let mut list = Vec::new();
    for node in state.dendrite_graph.by_tier(1) {
        list.push(MemoryNodeDto {
            id: node.id,
            label: node.title,
            content: node.content,
            node_type: "Fact".to_string(),
        });
    }
    Json(list)
}

#[derive(Serialize)]
struct MeshStatusResponse {
    online: bool,
    identity_hash: Option<String>,
}

async fn get_mesh_status(State(state): State<AppState>) -> Json<MeshStatusResponse> {
    if let Some(ref mesh) = state.mesh {
        Json(MeshStatusResponse {
            online: true,
            identity_hash: mesh.identity_hash.clone(),
        })
    } else {
        Json(MeshStatusResponse {
            online: false,
            identity_hash: None,
        })
    }
}

#[derive(Deserialize)]
struct MatrixParams {
    topic: String,
}

async fn get_matrix_results(
    Query(params): Query<MatrixParams>,
    State(state): State<AppState>,
) -> Json<paraclea_core::matrix::MatrixResult> {
    let reader_guard = state.bible_reader.read().await;
    let lib = state.library.read().await;
    if let Some(ref r) = *reader_guard {
        let res = paraclea_core::matrix::TopicMatrixEngine::build_matrix(&params.topic, r, &lib);
        Json(res)
    } else {
        let dummy_reader = BibleReader {
            books: Vec::new(),
            raw_data: None,
        };
        let res = paraclea_core::matrix::TopicMatrixEngine::build_matrix(&params.topic, &dummy_reader, &lib);
        Json(res)
    }
}

async fn get_mesh_mailbox(State(state): State<AppState>) -> Json<Vec<paraclea_core::mesh::MeshMessage>> {
    if let Some(ref mesh) = state.mesh {
        Json(mesh.read_mailbox())
    } else {
        Json(Vec::new())
    }
}

#[derive(Deserialize)]
struct SendMeshMsgPayload {
    recipient: String,
    content: String,
}

async fn send_mesh_message(
    State(state): State<AppState>,
    Json(payload): Json<SendMeshMsgPayload>,
) -> Json<Option<paraclea_core::mesh::MeshMessage>> {
    if let Some(ref mesh) = state.mesh {
        let res = mesh.send_message(&payload.recipient, &payload.content).ok();
        Json(res)
    } else {
        Json(None)
    }
}

#[derive(Serialize)]
struct DoctorResponse {
    ollama_online: bool,
    qdrant_online: bool,
    mesh_online: bool,
    mesh_identity: Option<String>,
    bibles_count: usize,
    languages_count: usize,
    library_books_count: usize,
    library_chapters_count: usize,
    dendrite_nodes_count: usize,
    active_model: String,
}

async fn run_doctor_checks(State(state): State<AppState>) -> Json<DoctorResponse> {
    let ollama_online = state.ollama.health_check().await.unwrap_or(false);
    let qdrant_online = state.qdrant.health_check().await;
    let mesh_online = state.mesh.is_some();
    let mesh_identity = state.mesh.as_ref().and_then(|m| m.identity_hash.clone());

    let languages = BibleReader::list_languages();
    let languages_count = languages.len();
    let bibles_count: usize = languages.iter().map(|l| BibleReader::list_translations_for_lang(&l.code).len()).sum();

    let lib = state.library.read().await;
    let library_books_count = lib.books.len();
    let library_chapters_count: usize = lib.books.iter().map(|b| b.chapters.len()).sum();

    let dendrite_nodes_count = state.dendrite_graph.by_tier(1).len()
        + state.dendrite_graph.by_tier(2).len()
        + state.dendrite_graph.by_tier(3).len()
        + state.dendrite_graph.by_tier(4).len();

    Json(DoctorResponse {
        ollama_online,
        qdrant_online,
        mesh_online,
        mesh_identity,
        bibles_count,
        languages_count,
        library_books_count,
        library_chapters_count,
        dendrite_nodes_count,
        active_model: state.ollama.model.clone(),
    })
}
