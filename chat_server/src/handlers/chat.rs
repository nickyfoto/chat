use axum::{response::IntoResponse, Extension};
use tracing::info;

use crate::User;

pub(crate) async fn list_chats_handler(Extension(user): Extension<User>) -> impl IntoResponse {
    info!("User: {:?}", user);
    "List chats"
}

pub(crate) async fn create_chat_handler() -> impl IntoResponse {
    "Create chat"
}

pub(crate) async fn update_chat_handler() -> impl IntoResponse {
    "Update chat"
}

pub(crate) async fn delete_chat_handler() -> impl IntoResponse {
    "Delete chat"
}
