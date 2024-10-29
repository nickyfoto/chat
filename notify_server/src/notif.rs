use std::{collections::HashSet, sync::Arc};

use crate::AppState;
use anyhow::Result;
use chat_core::{Chat, Message};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgListener;
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AppEvent {
    NewChat(Chat),
    AddToChat(Chat),
    RemoveFromChat(Chat),
    NewMessage(Message),
}

#[derive(Debug)]
struct Notification {
    event: Arc<AppEvent>,
    user_ids: HashSet<u64>,
}

// pg_notify('chat_updated', json_build_object('op', TG_OP, 'old', OLD, 'new', NEW)::text);
#[derive(Debug, Serialize, Deserialize)]
struct ChatUpdated {
    op: String,
    old: Option<Chat>,
    new: Option<Chat>,
}

// pg_notify('chat_message_created', row_to_json(NEW)::text);
#[derive(Debug, Serialize, Deserialize)]
struct ChatMessageCreated {
    message: Message,
    members: Vec<i64>,
}

pub async fn setup_pg_listener(state: AppState) -> Result<()> {
    // let mut listener = PgListener::connect("postgres://postgres:postgres@localhost/chat").await?;
    let mut listener = PgListener::connect(&state.config.server.db_url).await?;
    listener.listen("chat_updated").await?;
    listener.listen("chat_message_created").await?;

    let mut stream = listener.into_stream();
    tokio::spawn(async move {
        while let Some(Ok(notification)) = stream.next().await {
            info!("Received notification: {:?}", notification);
            let notification = Notification::load(notification.channel(), notification.payload())?;
            let users = &state.users;
            for user_id in notification.user_ids {
                if let Some(tx) = users.get(&user_id) {
                    info!("Sending notification to user: {}", user_id);
                    if let Err(e) = tx.send(notification.event.clone()) {
                        info!("Failed to send notification to user: {}", e);
                    }
                }
            }
        }
        Ok::<_, anyhow::Error>(())
    });
    Ok(())
}

impl Notification {
    fn load(r#type: &str, payload: &str) -> Result<Self> {
        match r#type {
            "chat_updated" => {
                let payload: ChatUpdated = serde_json::from_str(payload)?;
                info!("ChatUpdated: {:?}", payload);
                let user_ids =
                    get_affected_chat_user_ids(payload.old.as_ref(), payload.new.as_ref());
                let event = match payload.op.as_str() {
                    "INSERT" => AppEvent::NewChat(payload.new.expect("new chat not found")),
                    "UPDATE" => AppEvent::AddToChat(payload.new.expect("new chat not found")),
                    "DELETE" => AppEvent::RemoveFromChat(payload.old.expect("old chat not found")),
                    _ => return Err(anyhow::anyhow!("Unknown operation: {}", payload.op)),
                };
                Ok(Self {
                    event: Arc::new(event),
                    user_ids,
                })
            }
            "chat_message_created" => {
                let payload: ChatMessageCreated = serde_json::from_str(payload)?;
                let user_ids = payload.members.iter().map(|v| *v as u64).collect();
                Ok(Self {
                    event: Arc::new(AppEvent::NewMessage(payload.message)),
                    user_ids,
                })
            }
            _ => Err(anyhow::anyhow!("Unknown notification type: {}", r#type)),
        }
    }
}

fn get_affected_chat_user_ids(old: Option<&Chat>, new: Option<&Chat>) -> HashSet<u64> {
    match (old, new) {
        (Some(old), Some(new)) => {
            // diff old/new members, if identical, no need to notify, otherwise notify the union of both
            let old_user_ids: HashSet<_> = old.members.iter().map(|v| *v as u64).collect();
            let new_user_ids: HashSet<_> = new.members.iter().map(|v| *v as u64).collect();
            if old_user_ids == new_user_ids {
                HashSet::new()
            } else {
                old_user_ids.union(&new_user_ids).cloned().collect()
            }
        }
        (Some(old), None) => old.members.iter().map(|v| *v as u64).collect(),
        (None, Some(new)) => new.members.iter().map(|v| *v as u64).collect(),
        _ => HashSet::new(),
    }
}
