use serde::{Deserialize, Serialize};

pub use chat::CreateChat;
pub use message::{CreateMessage, ListMessages};
pub use user::{CreateUser, SigninUser};

mod chat;
mod file;
mod message;
mod user;
mod workspace;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatFile {
    pub ws_id: u64,
    pub ext: String,
    pub hash: String,
}
