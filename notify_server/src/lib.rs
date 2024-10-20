mod sse;

use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};

use anyhow::Result;

pub enum Event {
    NewChat(Chat),
    AddToChat(Chat),
    RemoveFromChat(Chat),
    NewMessage(Message),
}

const INDEX_HTML: &str = include_str!("../index.html");
use chat_core::{Chat, Message};
use futures::StreamExt;
use sqlx::postgres::PgListener;
use sse::sse_handler;
use tracing::info;

pub fn get_router() -> Router {
    Router::new()
        .route("/", get(index_handler))
        .route("/events", get(sse_handler))
}

pub async fn setup_pg_listener() -> Result<()> {
    let mut listener = PgListener::connect("postgres://postgres:postgres@localhost/chat").await?;
    listener.listen("chat_updated").await?;
    listener.listen("chat_message_created").await?;

    let mut stream = listener.into_stream();
    tokio::spawn(async move {
        while let Some(notification) = stream.next().await {
            info!("Received notification: {:?}", notification);
        }
    });
    Ok(())
}

async fn index_handler() -> impl IntoResponse {
    Html(INDEX_HTML)
}
