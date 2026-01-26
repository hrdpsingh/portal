use axum::{
    Json, Router,
    body::Body,
    extract::{Path, State},
    http::{StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use clap::Parser;
use local_ip_address::local_ip;
use std::{fs, net::SocketAddr, path::PathBuf, process, sync::Arc};
use tokio_util::io::ReaderStream;
use uuid::Uuid;

#[derive(Parser)]
struct Args {
    #[arg(long, required = true, num_args = 1..)]
    paths: Vec<PathBuf>,
}

#[derive(serde::Serialize, Clone)]
struct Files {
    uuid: Uuid,
    file_name: String,
    file_size: u64,
    file_path: PathBuf,
}

#[tokio::main]
async fn main() {
    let local_ip = match local_ip() {
        Ok(local_ip) => local_ip,
        Err(error) => {
            eprint!("Failed to get local IP address: {:?}", error);
            process::exit(1);
        }
    };

    let args = Args::parse();
    let mut canonical_paths = Vec::new();

    for path in args.paths {
        match fs::canonicalize(&path) {
            Ok(canonical_path) => canonical_paths.push(canonical_path),
            Err(e) => {
                eprintln!("Error: Could not canonicalize {:?}: {}", path, e);
                process::exit(1);
            }
        }
    }

    let namespace = Uuid::NAMESPACE_DNS;
    let mut files = Vec::new();

    for canonical_path in canonical_paths {
        let canonical_path_bytes = canonical_path.as_os_str().as_encoded_bytes();
        let uuid = Uuid::new_v5(&namespace, canonical_path_bytes);
        let file_name = match canonical_path.file_name().and_then(|s| s.to_str()) {
            Some(name) => name.to_string(),
            None => {
                eprintln!(
                    "Could not extract a valid UTF-8 file name from {:?}",
                    canonical_path
                );
                process::exit(1);
            }
        };

        let file_info = match fs::metadata(&canonical_path) {
            Ok(metadata) => metadata,
            Err(e) => {
                eprintln!("Error fetching details for {:?}: {}", canonical_path, e);
                process::exit(1);
            }
        };

        let file_size = file_info.len();
        let file_path = canonical_path;

        files.push(Files {
            uuid,
            file_name,
            file_size,
            file_path,
        });
    }

    let shared_state = Arc::new(files);
    let addr = SocketAddr::new(local_ip, 8000);
    let app = Router::new()
        .route("/", get(Html(include_str!("../index.html"))))
        .route("/metadata", get(get_metadata))
        .route("/download/{uuid}", get(download_file))
        .with_state(shared_state);

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind to address {}: {}", addr, e);
            process::exit(1);
        }
    };

    println!("Server running at http://{}", addr);
    println!("If you cannot connect, check your firewall rules for port 8000.");
    println!("Enter 'ctrl + c' to stop...");

    match axum::serve(listener, app).await {
        Ok(_) => println!("Server shut down successfully"),
        Err(e) => eprintln!("Server error: {}", e),
    }
}

async fn get_metadata(State(files): State<Arc<Vec<Files>>>) -> impl IntoResponse {
    Json(files.as_ref().clone())
}

async fn download_file(
    Path(uuid): Path<Uuid>,
    State(files): State<Arc<Vec<Files>>>,
) -> impl IntoResponse {
    let file_info = match files.iter().find(|f| f.uuid == uuid) {
        Some(f) => f,
        None => return Err((StatusCode::NOT_FOUND, "File not found")),
    };

    let file = match tokio::fs::File::open(&file_info.file_path).await {
        Ok(file) => file,
        Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "File system error")),
    };

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);
    let content_disposition = format!("attachment; filename=\"{}\"", file_info.file_name);

    let res = Response::builder()
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CONTENT_DISPOSITION, content_disposition)
        .header(header::CONTENT_LENGTH, file_info.file_size)
        .body(body)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to build response"))?; 

    Ok(res)
}
