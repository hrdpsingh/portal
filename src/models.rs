use clap::Parser;
use serde::Deserialize;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Parser)]
pub struct Args {
    #[arg(long, required = true, num_args = 1..)]
    pub paths: Vec<PathBuf>,

    #[arg(long, required = true)]
    pub password: String,
}

#[derive(serde::Serialize, Clone)]
pub struct FileInfo {
    pub uuid: Uuid,
    pub file_name: String,
    pub file_size: u64,
    pub file_path: PathBuf,
}

#[derive(Clone)]
pub struct AppState {
    pub metadata: Vec<FileInfo>,
    pub password: String,
    pub auth_token: String,
}

#[derive(Deserialize)]
pub struct LoginPayload {
    pub password: String,
}
