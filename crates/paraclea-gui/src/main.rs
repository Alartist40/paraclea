use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use colored::*;
use paraclea_core::{
    bible::{self, BibleReader},
    dendrite::{Dendrite, DendriteStore},
    library::LibraryEngine,
    mesh::ReticulumEngine,
    ollama::{ChatMessage, OllamaClient},
    persona::PersonaManager,
    qdrant::QdrantClient,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

const HTML_CONTENT: &str = include_str!("../public/index.html");

#[derive(Clone)]
struct AppState {
    ollama: Arc<OllamaClient>,
    persona: Arc<PersonaManager>,
    library: Arc<tokio::sync::RwLock<LibraryEngine>>,
    dendrite_store: Option<Arc<DendriteStore>>,
    mesh: Option<Arc<ReticulumEngine>>,
    qdrant: Arc<QdrantClient>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("{}", "╔══════════════════════════════════════════════════════════════╗".purple().bold());
    println!("{}", "║     PARACLEA AI ASSISTANT — DESKTOP APPLICATION SERVER       ║".yellow().bold());
    println!("{}", "╚══════════════════════════════════════════════════════════════╝".purple().bold());

    let persona_dir = std::env::var("HOME")
        .map(|h| PathBuf::from(h).join(".paraclea/persona"))
        .unwrap_or_else(|_| PathBuf::from("persona"));
    let persona = Arc::new(PersonaManager::new(persona_dir).unwrap_or_else(|_| PersonaManager { persona_dir: PathBuf::from("persona") }));

    let ollama = Arc::new(OllamaClient::new("http://localhost:11434", "ministral-3:3b")?);
    let qdrant = Arc::new(QdrantClient::new("http://localhost:6333")?);
    let library = Arc::new(tokio::sync::RwLock::new(LibraryEngine::load_auto()));
    let mesh = ReticulumEngine::new().ok().map(Arc::new);

    let dendrite_graph = Arc::new(Dendrite::new());
    let dendrite_store = std::env::var("HOME").ok().and_then(|h| {
        let db_path = PathBuf::from(h).join(".paraclea/dendrite.db");
        DendriteStore::open(&db_path).ok().map(Arc::new)
    });
    if let Some(ref store) = dendrite_store {
        let _ = store.load_all(&dendrite_graph);
    }

    let state = AppState {
        ollama,
        persona,
        library,
        dendrite_store,
        mesh,
        qdrant,
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
        .route("/api/memory", get(get_memory_nodes))
        .route("/api/mesh", get(get_mesh_status))
        .route("/api/doctor", get(run_doctor_checks))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 7860));
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

async fn get_bible_books() -> Json<Vec<BibleBookDto>> {
    let reader = BibleReader::load_auto().ok();
    let mut books = Vec::new();

    for &b in bible::OLD_TESTAMENT_BOOKS {
        let total_chapters = reader.as_ref().and_then(|r| r.get_chapter_count(b)).unwrap_or(1);
        books.push(BibleBookDto {
            name: b.to_string(),
            testament: "Old Testament".to_string(),
            total_chapters,
        });
    }

    for &b in bible::NEW_TESTAMENT_BOOKS {
        let total_chapters = reader.as_ref().and_then(|r| r.get_chapter_count(b)).unwrap_or(1);
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

async fn read_bible_chapter(Query(params): Query<ReadBibleParams>) -> Json<BibleChapterResponse> {
    let reader = if let Some(path) = bible::find_json_bible_file(&params.tag) {
        BibleReader::load_primary(&path).ok()
    } else {
        BibleReader::load_auto().ok()
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
) -> Json<ChatResponse> {
    let system_prompt = state.persona.build_system_prompt();
    let msgs = vec![
        ChatMessage { role: "system".to_string(), content: system_prompt },
        ChatMessage { role: "user".to_string(), content: payload.message },
    ];

    match state.ollama.chat(msgs).await {
        Ok(reply) => Json(ChatResponse { reply }),
        Err(err) => Json(ChatResponse {
            reply: format!("⚠️ Unable to connect to Ollama model: {}", err),
        }),
    }
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
    if let Some(ref store) = state.dendrite_store {
        let temp_graph = Dendrite::new();
        if store.load_all(&temp_graph).is_ok() {
            for node in temp_graph.by_tier(1) {
                list.push(MemoryNodeDto {
                    id: node.id,
                    label: node.title,
                    content: node.content,
                    node_type: "Fact".to_string(),
                });
            }
        }
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

    let mut dendrite_nodes_count = 0;
    if let Some(ref store) = state.dendrite_store {
        let temp_graph = Dendrite::new();
        if store.load_all(&temp_graph).is_ok() {
            dendrite_nodes_count = temp_graph.by_tier(1).len() + temp_graph.by_tier(2).len() + temp_graph.by_tier(3).len() + temp_graph.by_tier(4).len();
        }
    }

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
