use axum::{body::Body, http::Request, response::IntoResponse};
use mongodb::bson::uuid;

use crate::request_context::X_REQUEST_ID;

pub async fn request_id_middleware(
    mut req: Request<Body>,
    next: axum::middleware::Next,
) -> impl IntoResponse {
    // 生成或获取请求ID
    let req_id = uuid::Uuid::new().to_string();
    req.headers_mut()
        .insert(X_REQUEST_ID, req_id.parse().unwrap());

    // println!("请求ID: {}", req_id);

    next.run(req).await
}
