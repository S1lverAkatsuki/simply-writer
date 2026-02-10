use std::sync::{Arc, Mutex};

use axum::{Json, Router, extract::State, response::Html, routing::get};
use clap::Parser;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Data {
    content: String,
    title: String,
    saved: bool,
}

#[derive(Clone)]
struct AppState {
    file_path: Arc<Mutex<Option<String>>>,
}

#[derive(Parser, Debug)]
#[command(version, about = "A simply web ui note")]
struct Args {
    /// Path to the text file
    path: Option<String>,

    #[arg(short, long, default_value_t = 3000)]
    /// Port to listen on
    port: u16,
}

async fn load(State(state): State<AppState>) -> Json<Data> {
    let maybe_path = state.file_path.lock().unwrap().clone();

    match maybe_path {
        Some(path) => {
            let content = tokio::fs::read_to_string(&path).await.unwrap_or_default();
            let title = std::path::Path::new(&path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or(path);
            Json(Data {
                content,
                title,
                saved: true,
            })
        }
        None => Json(Data {
            content: String::new(),
            title: "Untitled (Not Saved)".to_string(),
            saved: false,
        }),
    }
}

async fn save(State(state): State<AppState>, axum::Json(payload): axum::Json<Data>) -> Json<Data> {
    let current_path = {
        let mut path_lock = state.file_path.lock().unwrap();

        if path_lock.is_none() {
            if let Some(path) = FileDialog::new()
                .add_filter("Plaintext", &["txt"])
                .add_filter("Markdown", &["md"])
                .save_file()
            {
                *path_lock = Some(path.to_string_lossy().to_string());
            } else {
                // 用户取消
                return axum::Json(payload);
            }
        }
        path_lock.as_ref().unwrap().clone()
    };

    tokio::fs::write(&current_path, &payload.content)
        .await
        .expect("Failed to write file");

    let title = std::path::Path::new(&current_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or(current_path);

    Json(Data {
        content: payload.content,
        title,
        saved: true,
    })
}

async fn status() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let state = AppState {
        file_path: Arc::new(Mutex::new(args.path)),
    };

    let app = Router::new()
        .route("/api/status", get(status))
        .route("/api/content", get(load).post(save))
        .route("/", get(|| async { Html(include_str!("index.html")) }))
        .with_state(state);

    let addr = format!("localhost:{}", args.port);
    println!("Service run at: http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
