use axum::{
    extract::State,
    http::{Method, StatusCode},
    response::IntoResponse,
};

pub(super) async fn handle(State(app): State<super::App>, method: Method) -> impl IntoResponse {
    if method != Method::POST {
        return StatusCode::NOT_FOUND.into_response();
    }
    axum::Json(app.kernel.version().clone()).into_response()
}
