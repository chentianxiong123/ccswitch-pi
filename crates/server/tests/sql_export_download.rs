use std::sync::Arc;

use axum::{
    body::{to_bytes, Body},
    http::{header, Request, StatusCode},
    routing::get,
    Router,
};
use cc_switch::{AppState, Database};
use cc_switch_core::CoreContext;
use cc_switch_server::{api::export_sql_download_handler, create_event_bus, ServerState};
use tower::util::ServiceExt;

#[tokio::test]
async fn sql_download_returns_attachment_headers_and_sql_body() {
    let db = Arc::new(Database::memory().expect("in-memory database"));
    let event_bus = create_event_bus(8);
    let state = ServerState::new(event_bus);

    let app = Router::new()
        .route("/api/export-config", get(export_sql_download_handler))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/export-config")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("application/sql; charset=utf-8")
    );
    let disposition = response
        .headers()
        .get(header::CONTENT_DISPOSITION)
        .and_then(|value| value.to_str().ok())
        .expect("content disposition");
    assert!(disposition.starts_with("attachment; filename=\"cc-switch-export-"));
    assert!(disposition.ends_with(".sql\""));

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body bytes");
    let sql = String::from_utf8(body.to_vec()).expect("utf8 sql");
    assert!(sql.starts_with("-- CC Switch SQLite 导出"));
}