use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};

use crate::{AppError, AppState, CreateChat, User};

pub(crate) async fn create_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Json(input): Json<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.create_chat(input, user.ws_id as _).await?;
    Ok((StatusCode::CREATED, Json(chat)))
}

pub(crate) async fn list_chats_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let chats = state.fetch_chats(user.ws_id as _).await?;
    Ok((StatusCode::OK, Json(chats)))
}

pub(crate) async fn get_chat_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.get_chat_by_id(id as _).await?;
    match chat {
        Some(chat) => Ok((StatusCode::OK, Json(chat))),
        None => Err(AppError::NotFound(format!("chat id {id}"))),
    }
}

pub(crate) async fn update_chat_handler() -> impl IntoResponse {
    "Update chat"
}

pub(crate) async fn delete_chat_handler() -> impl IntoResponse {
    "Delete chat"
}
