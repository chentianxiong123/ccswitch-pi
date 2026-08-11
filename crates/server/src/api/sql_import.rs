use std::sync::Arc;

use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::{json, Value};

use crate::state::ServerState;

pub const MAX_SQL_UPLOAD_BYTES: usize = 200 * 1024 * 1024;

fn respond(status: StatusCode, body: Value) -> impl IntoResponse {
    (status, Json(body))
}

pub async fn import_sql_upload_handler(
    State(state): State<Arc<ServerState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {

    let field = match multipart.next_field().await {
        Ok(Some(field)) => field,
        Ok(None) => {
            return respond(
                StatusCode::BAD_REQUEST,
                json!({
                    "success": false,
                    "message": "缺少上传文件"
                }),
            );
        }
        Err(err) => {
            return respond(
                StatusCode::BAD_REQUEST,
                json!({
                    "success": false,
                    "message": format!("读取上传内容失败: {err}")
                }),
            );
        }
    };

    let bytes = match field.bytes().await {
        Ok(bytes) => bytes,
        Err(err) => {
            return respond(
                StatusCode::BAD_REQUEST,
                json!({
                    "success": false,
                    "message": format!("读取上传文件失败: {err}")
                }),
            );
        }
    };

    match cc_switch_core::import_config_from_sql_bytes(&state.core, bytes.as_ref()) {
        Ok(result) => respond(StatusCode::OK, result),
        Err(err) => respond(
            StatusCode::BAD_REQUEST,
            json!({
                "success": false,
                "message": err
            }),
        ),
    }
}