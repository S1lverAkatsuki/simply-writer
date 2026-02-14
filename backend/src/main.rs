use std::sync::Arc;

use tokio::sync::OnceCell;

use anyhow::Result;
use axum::{Json, Router, extract::State, http::StatusCode, response::Html, routing::get};
use clap::{Parser, ValueEnum};
use encoding_rs::{GBK, UTF_8};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
enum Encodes {
    #[value(name = "utf-8")]
    Utf8,
    #[value(name = "gbk")]
    Gbk,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Data {
    content: String,
    title: String,
    saved: bool,
}

#[derive(Clone)]
struct AppState {
    file_path: Arc<OnceCell<String>>,
    encoding: Arc<OnceCell<Encodes>>,
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

    #[arg(short, long, default_value = "utf-8")]
    // Use which encode to create / open file
    encoding: Encodes,
}

async fn read_with_encoding(path: &str, encoding: &Encodes) -> Result<String> {
    let bytes = tokio::fs::read(path).await?;

    let encoder: &'static encoding_rs::Encoding = match encoding {
        Encodes::Utf8 => UTF_8,
        Encodes::Gbk => GBK,
    };

    let (decoded, _, _has_errors) = encoder.decode(&bytes);

    // 如果这里丢编码错误，编码对不上就返回空字符串 (unwrap_or_default) ，如果按下保存就会顶掉原本的信息
    // if has_errors {
    //     anyhow::bail!("Failed to decode file at {} using {:?}", path, encoding);
    // }

    Ok(decoded.into_owned())
}

async fn write_with_encoding(path: &str, content: &str, encoding: &Encodes) -> Result<()> {
    let encoder = match encoding {
        Encodes::Utf8 => encoding_rs::UTF_8,
        Encodes::Gbk => encoding_rs::GBK,
    };

    let (encoded_bytes, _, has_errors) = encoder.encode(content);

    if has_errors {
        anyhow::bail!(
            "Content contains characters that cannot be encoded in {:?}",
            encoding
        );
    }

    tokio::fs::write(path, &encoded_bytes).await?;

    Ok(())
}

async fn load(State(state): State<AppState>) -> Json<Data> {
    let maybe_path = state.file_path.get();
    let encode = state.encoding.get().clone().unwrap_or(&Encodes::Utf8);

    match maybe_path {
        Some(path) => {
            match read_with_encoding(path, &encode).await {
                Ok(content) => {
                    let title = std::path::Path::new(path)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| path.clone());

                    Json(Data {
                        content,
                        title,
                        saved: true,
                    })
                }
                Err(e) => {
                    // IO 失败处理：比如文件被占用或消失了
                    eprintln!("Failed to read file {}: {}", path, e);
                    Json(Data {
                        content: format!("Error reading file: {}", e),
                        title: "Error".into(),
                        saved: false, // 既然读都读不到，肯定不能算 saved
                    })
                }
            }
        }
        None => {
            // 初次打开
            Json(Data {
                content: String::new(),
                title: DEFAULT_FILE_NAME.to_string(),
                saved: false,
            })
        }
    }
}

async fn save(State(state): State<AppState>, Json(payload): axum::Json<Data>) -> Json<Data> {
    let current_path = if let Some(path) = state.file_path.get() {
        path.clone()
    } else {
        if let Some(path) = FileDialog::new()
            .add_filter("Plaintext", &["txt"])
            .add_filter("Markdown", &["md"])
            .set_file_name(DEFAULT_FILE_NAME)
            .save_file()
        {
            let path_str = path.to_string_lossy().to_string();

            let final_path = state
                .file_path
                .get_or_init(|| async { path_str.clone() })
                .await;

            println!("New file has saved at {}", final_path);
            final_path.clone()
        } else {
            // 用户取消了对话框
            return Json(Data {
                content: payload.content,
                title: payload.title,
                saved: false,
            });
        }
    };

    let encoding = state.encoding.get().unwrap_or(&Encodes::Utf8);

    let save_res = write_with_encoding(&current_path, &payload.content, encoding).await;

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

    // 还得可选初始化，没东西就别碰 OnceCell
    let file_path = Arc::new(OnceCell::new());
    if let Some(p) = args.path {
        let _ = file_path.set(p);
    }

    let encoding = Arc::new(OnceCell::new());
    let _ = encoding.set(args.encoding);

    let state = AppState {
        file_path,
        encoding,
    };

    let app = Router::new()
        .route("/api/status", get(status))
        .route("/api/content", get(load).post(save))
        .route("/", get(|| async { Html(INDEX_HTML) }))
        .with_state(state);

    println!("Encoding: {:?}", args.encoding);

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
