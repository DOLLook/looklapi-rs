use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

/// app响应结果
#[derive(Serialize, Deserialize, Clone)]
pub struct AppResponse<T> {
    #[serde(rename = "IsSuccess")]
    pub is_success: bool,
    #[serde(rename = "ErrorCode")]
    pub error_code: i32,
    #[serde(rename = "ErrorMsg")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_msg: Option<String>,
    #[serde(rename = "Result")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<T>,
}

impl<T> AppResponse<T> {
    pub fn new(result: T) -> Self {
        Self {
            is_success: true,
            error_code: 0,
            error_msg: None,
            result: Some(result),
        }
    }

    // pub fn fail(err: &'static dyn Error) -> Self {
    //     let mut e = Self {
    //         is_success: false,
    //         error_code: -1,
    //         error_msg: Some(err.to_string()),
    //         result: None,
    //     };

    //     if let Some(ae) = err.downcast_ref::<AppError>() {
    //         e.error_code = ae.code();
    //     }
    //     e
    // }
}

// Tell axum how to convert `ResponseResult` into a response.
impl<T> IntoResponse for AppResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        axum::Json(self).into_response()
    }
}
