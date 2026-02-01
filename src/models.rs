use serde::Deserialize;
use std::path::PathBuf;
use uuid::Uuid;

pub struct InputData {
    pub password: String,
    pub paths: Vec<PathBuf>,
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
