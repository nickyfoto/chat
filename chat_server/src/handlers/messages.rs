use axum::{
    extract::{Multipart, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
use tokio::fs;
use tracing::{info, warn};

use crate::{models::ChatFile, AppError, AppState, CreateMessage, ListMessages};
use chat_core::User;

pub(crate) async fn list_messages_handler(
    State(state): State<AppState>,
    Path(chat_id): Path<i64>,
    Query(input): Query<ListMessages>,
) -> Result<impl IntoResponse, AppError> {
    let messages = state.list_messages(input, chat_id).await?;
    Ok(Json(messages))
}

pub(crate) async fn send_message_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path(chat_id): Path<i64>,
    Json(input): Json<CreateMessage>,
) -> Result<impl IntoResponse, AppError> {
    let message = state.create_message(input, chat_id, user.id as _).await?;
    Ok((StatusCode::CREATED, Json(message)))
}

pub(crate) async fn file_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path((ws_id, path)): Path<(i64, String)>,
) -> Result<impl IntoResponse, AppError> {
    if user.ws_id != ws_id {
        return Err(AppError::NotFound(
            "File doesn't exist or you don't have permission".to_string(),
        ));
    }
    let base_dir = &state.config.server.base_dir.join(ws_id.to_string());
    let path = base_dir.join(path);
    info!("File path: {:?}", path);
    if !path.exists() {
        return Err(AppError::NotFound("File doesn't exist".to_string()));
    }
    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    let body = fs::read(path).await?;
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", mime.to_string().parse()?);
    Ok((headers, body))
}

pub(crate) async fn upload_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let ws_id = user.ws_id as u64;
    let base_dir = &state.config.server.base_dir;
    let mut files = vec![];
    while let Some(field) = multipart.next_field().await.unwrap() {
        let filename = field.file_name().map(|name| name.to_string());
        let (Some(filename), Ok(data)) = (filename, field.bytes().await) else {
            warn!("Failed to read multipart field");
            continue;
        };
        let file = ChatFile::new(ws_id, &filename, &data);
        let path = file.path(base_dir);
        if path.exists() {
            info!("File {} already exists {:?}", filename, path);
        } else {
            fs::create_dir_all(path.parent().expect("file path parent should exist")).await?;
            fs::write(path, data).await?;
        }
        files.push(file.url());
    }
    Ok(Json(files))
}
