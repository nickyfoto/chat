mod config;
mod error;
mod notif;
mod sse;

use std::{ops::Deref, sync::Arc};

use axum::{
    middleware::from_fn_with_state,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};

use anyhow::Result;
pub use config::AppConfig;

pub enum Event {
    NewChat(Chat),
    AddToChat(Chat),
    RemoveFromChat(Chat),
    NewMessage(Message),
}

const INDEX_HTML: &str = include_str!("../index.html");
use chat_core::{
    middlewares::{verify_token, TokenVerify},
    Chat, DecodingKey, Message, User,
};
use dashmap::DashMap;
use error::AppError;
pub use notif::setup_pg_listener;
use notif::AppEvent;
use sse::sse_handler;
use tokio::sync::broadcast;

pub type UserMap = Arc<DashMap<u64, broadcast::Sender<Arc<AppEvent>>>>;

#[derive(Clone)]
pub struct AppState(Arc<AppStateInner>);

pub struct AppStateInner {
    pub config: AppConfig,
    users: UserMap,
    dk: DecodingKey,
}

pub fn get_router() -> (Router, AppState) {
    let config = AppConfig::load().expect("Failed to load config");
    let state = AppState::new(config);
    let app = Router::new()
        .route("/events", get(sse_handler))
        .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
        .route("/", get(index_handler))
        .with_state(state.clone());
    (app, state)
}

async fn index_handler() -> impl IntoResponse {
    Html(INDEX_HTML)
}

impl AppState {
    pub fn new(config: AppConfig) -> AppState {
        let dk = DecodingKey::load(&config.auth.pk).expect("Failed to load decoding key");
        let users = Arc::new(DashMap::new());
        Self(Arc::new(AppStateInner { config, users, dk }))
    }
}

impl TokenVerify for AppState {
    type Error = AppError;
    fn verify(&self, token: &str) -> Result<User, Self::Error> {
        Ok(self.dk.verify(token)?)
    }
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
