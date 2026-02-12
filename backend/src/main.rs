use std::sync::Arc;
use tokio::sync::Mutex;

use axum::{Json, Router, extract::State, http::StatusCode, response::Html, routing::get};
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

const INDEX_HTML: &str = include_str!("../../frontend-web/dist/index.html");
const DEFAULT_FILE_NAME: &str = "Untitled";

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
    let maybe_path = state.file_path.lock().await.clone();

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
            title: DEFAULT_FILE_NAME.to_string(),
            saved: false,
        }),
    }
}

async fn save(State(state): State<AppState>, Json(payload): axum::Json<Data>) -> Json<Data> {
    let current_path = {
        let mut path_lock = state.file_path.lock().await;

        if path_lock.is_none() {
            if let Some(path) = FileDialog::new()
                .add_filter("Plaintext", &["txt"])
                .add_filter("Markdown", &["md"])
                .set_file_name(DEFAULT_FILE_NAME)
                .save_file()
            {
                *path_lock = Some(path.to_string_lossy().to_string());
                println!("New file has saved at {}", path.to_string_lossy());
            } else {
                // 关闭了保存文件的窗口
                return Json(payload);
            }
        }
        path_lock.as_ref().unwrap().clone()
    };

    let save_res = tokio::fs::write(&current_path, &payload.content).await;

    let title = std::path::Path::new(&current_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or(DEFAULT_FILE_NAME.into());

    match save_res {
        Ok(_) => Json(Data {
            content: payload.content,
            title,
            saved: true,
        }),
        Err(e) => {
            eprintln!("Error writing file: {}", e);
            Json(Data {
                content: payload.content,
                title,
                saved: false,
            })
        }
    }
}

async fn status() -> StatusCode {
    StatusCode::OK
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
        .route("/", get(|| async { Html(INDEX_HTML) }))
        .with_state(state);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], args.port));
    println!("Service run at: http://{}", addr);

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::AddrInUse {
                eprintln!(
                    "Error: Address {} has already been used. Please use another available port.",
                    addr
                );
            } else {
                eprintln!("Address binding error ({}): {}", addr, e);
            }

            press_btn_continue::wait("Press any key to continue...").unwrap();

            std::process::exit(1);
        }
    };

    axum::serve(listener, app).await.unwrap();
}
