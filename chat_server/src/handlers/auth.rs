use axum::response::IntoResponse;

pub(crate) async fn signin_handler() -> impl IntoResponse {
    "Signin"
}

pub(crate) async fn signup_handler() -> impl IntoResponse {
    "Signup"
}
