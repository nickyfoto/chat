use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{AppError, AppState, Message};

use super::ChatFile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMessages {
    pub last_id: Option<i64>,
    pub limit: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessage {
    pub content: String,
    pub files: Vec<String>,
}

impl AppState {
    pub async fn create_message(
        &self,
        input: CreateMessage,
        chat_id: i64,
        user_id: u64,
    ) -> Result<Message, AppError> {
        let base_dir = &self.config.server.base_dir;

        if input.content.is_empty() {
            return Err(AppError::CreateMessageError(
                "content cannot be empty".to_string(),
            ));
        }

        // verify file exists
        for s in &input.files {
            let file = ChatFile::from_str(s)?;
            if !file.path(base_dir).exists() {
                return Err(AppError::CreateMessageError(format!(
                    "file {} doesn't exist",
                    s
                )));
            }
        }

        let message = sqlx::query_as(
            r#"
            INSERT INTO messages (chat_id, sender_id, content, files)
            VALUES ($1, $2, $3, $4)
            RETURNING id, chat_id, sender_id, content, files, created_at
            "#,
        )
        .bind(chat_id)
        .bind(user_id as i64)
        .bind(input.content)
        .bind(&input.files)
        .fetch_one(&self.pool)
        .await?;

        Ok(message)
    }

    pub async fn list_messages(
        &self,
        input: ListMessages,
        chat_id: i64,
    ) -> Result<Vec<Message>, AppError> {
        let last_id = input.last_id.unwrap_or(i64::MAX);

        let messages = sqlx::query_as(
            r#"
            SELECT id, chat_id, sender_id, content, files, created_at
            FROM messages
            WHERE chat_id = $1
            AND id < $2
            ORDER BY id DESC
            LIMIT $3
            "#,
        )
        .bind(chat_id)
        .bind(last_id)
        .bind(input.limit as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(messages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn create_message_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = CreateMessage {
            content: "hello".to_string(),
            files: vec![],
        };
        let message = state
            .create_message(input, 1, 1)
            .await
            .expect("create message failed");
        assert_eq!(message.content, "hello");

        // invliad files should fail
        let input = CreateMessage {
            content: "hello".to_string(),
            files: vec!["1".to_string()],
        };

        let err = state.create_message(input, 1, 1).await.unwrap_err();
        assert_eq!(err.to_string(), "Invalid chat file path: 1");

        // valid files should work
        let url = upload_dummy_file(&state)?;
        let input = CreateMessage {
            content: "hello".to_string(),
            files: vec![url],
        };

        let message = state
            .create_message(input, 1, 1)
            .await
            .expect("create message failed");
        assert_eq!(message.content, "hello");
        assert_eq!(message.files.len(), 1);
        Ok(())
    }

    #[tokio::test]
    async fn list_messages_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = ListMessages {
            last_id: None,
            limit: 6,
        };
        let messages = state.list_messages(input, 1).await?;
        assert_eq!(messages.len(), 6);

        let last_id = messages.last().expect("last message should exist").id;

        let input = ListMessages {
            last_id: Some(last_id),
            limit: 6,
        };

        let messages = state.list_messages(input, 1).await?;
        assert_eq!(messages.len(), 4);
        Ok(())
    }

    fn upload_dummy_file(state: &AppState) -> Result<String> {
        let file = ChatFile::new(1, "test.txt", b"hello world");
        let path = file.path(&state.config.server.base_dir);
        std::fs::create_dir_all(path.parent().expect("file path parent should exist"))?;
        std::fs::write(&path, b"hello world")?;
        let url = file.url();
        Ok(url)
    }
}
