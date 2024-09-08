use axum::response::IntoResponse;

pub(crate) async fn list_messages_handler() -> impl IntoResponse {
    "List messages"
}

pub(crate) async fn send_message_handler() -> impl IntoResponse {
    "Send message"
}
