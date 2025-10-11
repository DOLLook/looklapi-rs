use std::{backtrace::Backtrace, error::Error, fmt::Display};

use axum::response::{IntoResponse, Response};

use crate::app::response::AppResponse;

#[derive(Debug)]
pub struct AppError {
    code: i32,
    message: String,
    backtrace: Backtrace,
}

impl AppError {
    pub fn new(message: &str) -> Self {
        AppError {
            code: -1,
            message: message.to_string(),
            backtrace: Backtrace::capture(),
        }
    }

    pub fn new_with_errcode(code: i32, message: &str) -> Self {
        AppError {
            code,
            message: message.to_string(),
            backtrace: Backtrace::capture(),
        }
    }

    // 获取错误码
    pub fn code(&self) -> i32 {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    // 获取堆栈跟踪
    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "code:{}, {}", self.code, self.message)
    }
}

impl Error for AppError {}

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let rsp = AppResponse::<()> {
            is_success: false,
            error_code: self.code(),
            error_msg: Some(self.message),
            result: None,
        };
        rsp.into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        let message = err.to_string();
        if let Ok(app_err) = err.downcast::<AppError>() {
            return app_err;
        }

        AppError::new_with_errcode(-1, &message)
    }
}
