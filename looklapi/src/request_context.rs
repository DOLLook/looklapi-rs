use axum::http::{HeaderMap, HeaderName};

pub const X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");
pub const HTTP_REQUEST_HEADER: HeaderName = HeaderName::from_static("http-request-header");
pub const HTTP_REQUEST_EXTENSIONS: HeaderName = HeaderName::from_static("http-request-extensions");

/// 请求上下文
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub header: HeaderMap,
    pub login_info: Option<LoginInfo>,
}

/// 登录信息
#[derive(Debug, Clone)]
pub enum LoginInfo {
    User(()),
    Admin(()),
}
