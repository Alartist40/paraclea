use colored::*;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

const HTML_CONTENT: &str = include_str!("../public/index.html");

fn main() -> anyhow::Result<()> {
    println!("{}", "╔══════════════════════════════════════════════════════════════╗".purple().bold());
    println!("{}", "║     PARACLEA AI ASSISTANT — DESKTOP APPLICATION SERVER       ║".yellow().bold());
    println!("{}", "╚══════════════════════════════════════════════════════════════╝".purple().bold());
    println!("{}", "  Local Desktop GUI Server starting at http://127.0.0.1:7860...".yellow().bold());

    let listener = TcpListener::bind("127.0.0.1:7860")?;
    println!("{}", "  ✓ Paraclea Desktop Application running! Opening browser...".green().bold());

    // Open browser window
    let _ = open::that("http://127.0.0.1:7860");

    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            thread::spawn(move || {
                let mut buffer = [0; 1024];
                let _ = stream.read(&mut buffer);

                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    HTML_CONTENT.len(),
                    HTML_CONTENT
                );

                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            });
        }
    }

    Ok(())
}
