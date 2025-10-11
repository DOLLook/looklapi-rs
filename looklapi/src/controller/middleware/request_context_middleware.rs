use axum::{body::Body, http::Request, response::IntoResponse};

use crate::request_context::RequestContext;

/// 存储请求上下文
pub async fn request_ctx_middleware(
    mut req: Request<Body>,
    next: axum::middleware::Next,
) -> impl IntoResponse {
    let header = req.headers().clone();
    let ctx = RequestContext {
        header,
        login_info: None,
    };

    req.extensions_mut().insert(ctx);

    next.run(req).await
}

/// 登录验证
pub async fn login_validator_middleware(
    mut req: Request<Body>,
    next: axum::middleware::Next,
) -> impl IntoResponse {
    // todo 登录验证
    next.run(req).await
}

/// 管理员验证
pub async fn manager_validator_middleware(
    mut req: Request<Body>,
    next: axum::middleware::Next,
) -> impl IntoResponse {
    // todo 登录验证
    next.run(req).await
}
