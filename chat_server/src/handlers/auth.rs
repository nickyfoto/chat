use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{
    models::{CreateUser, SigninUser},
    AppError, AppState, ErrorOutput,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthOutput {
    token: String,
}

pub(crate) async fn signin_handler(
    State(state): State<AppState>,
    Json(input): Json<SigninUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = state.verify(&input).await?;
    match user {
        Some(user) => {
            let token = state.ek.sign(user)?;
            let body = Json(AuthOutput { token });
            Ok((StatusCode::OK, body).into_response())
        }
        None => {
            let body = Json(ErrorOutput::new("Invalid email or password"));
            Ok((StatusCode::FORBIDDEN, body).into_response())
        }
    }
}

pub(crate) async fn signup_handler(
    State(state): State<AppState>,
    Json(input): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = state.create_user(&input).await?;
    let token = state.ek.sign(user)?;
    let body = Json(AuthOutput { token });
    Ok((StatusCode::CREATED, body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn signup_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = CreateUser::new("acme", "Tian Chen", "tye@acme.org", "password");
        let res = signup_handler(State(state), Json(input))
            .await?
            .into_response();
        assert_eq!(res.status(), StatusCode::CREATED);
        let body = res.into_body().collect().await?.to_bytes();
        let res: AuthOutput = serde_json::from_slice(&body)?;
        assert_ne!(res.token, "");
        Ok(())
    }

    #[tokio::test]
    async fn signup_duplicate_user_should_409() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = CreateUser::new("acme", "Tyr Chen", "tchen@acme.org", "123456");
        let res = signup_handler(State(state), Json(input))
            .await
            .into_response();
        assert_eq!(res.status(), StatusCode::CONFLICT);
        let body = res.into_body().collect().await?.to_bytes();
        let res: ErrorOutput = serde_json::from_slice(&body)?;
        assert_eq!(res.error, "email already exists: tchen@acme.org");
        Ok(())
    }

    #[tokio::test]
    async fn signin_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let email = "tchen@acme.org";
        let password = "123456";
        let input = SigninUser::new(email, password);
        let res = signin_handler(State(state), Json(input))
            .await?
            .into_response();
        assert_eq!(res.status(), StatusCode::OK);
        let body = res.into_body().collect().await?.to_bytes();
        let res: AuthOutput = serde_json::from_slice(&body)?;
        assert_ne!(res.token, "");
        Ok(())
    }

    #[tokio::test]
    async fn signin_with_non_exist_user_should_403() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let email = "tchen1@acme.org";
        let password = "123456";
        let input = SigninUser::new(email, password);
        let res = signin_handler(State(state), Json(input))
            .await
            .into_response();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
        let body = res.into_body().collect().await?.to_bytes();
        let res: ErrorOutput = serde_json::from_slice(&body)?;
        assert_eq!(res.error, "Invalid email or password");
        Ok(())
    }
}
