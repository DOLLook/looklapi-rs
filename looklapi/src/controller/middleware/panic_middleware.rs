use std::panic::AssertUnwindSafe;

use axum::{body::Body, http::Request, middleware::Next, response::IntoResponse};
use futures::FutureExt;

use crate::app::AppResponse;

pub async fn panic_handler(req: Request<Body>, next: Next) -> impl IntoResponse {
    let uri = req.uri().to_string();
    let future = AssertUnwindSafe(next.run(req)).catch_unwind();
    match future.await {
        Ok(rsp) => rsp,
        Err(err) => {
            // // 记录 panic 日志（使用你现有的日志格式）
            // tracing::error!("Request handler panicked");
            if let Some(s) = err.downcast_ref::<String>() {
                println!("Request handler panicked: {}, uri: {}", s, uri)
            } else if let Some(s) = err.downcast_ref::<&str>() {
                println!("Request handler panicked: {}, uri: {}", s, uri)
            } else {
                println!("Request handler panicked, uri: {}", uri);
            }

            let rsp = AppResponse::<()> {
                is_success: false,
                error_code: -1,
                error_msg: Some("system error".to_string()),
                result: None,
            };
            rsp.into_response()
        }
    }
}

// /// The default `ResponseForPanic` used by `CatchPanic`.
// ///
// /// It will log the panic message and return a `500 Internal Server` error response with an empty
// /// body.
// #[derive(Debug, Default, Clone, Copy)]
// #[non_exhaustive]
// pub struct PanicMiddleware;

// impl ResponseForPanic for PanicMiddleware {
//     type ResponseBody = AppResponse<()>;
//     fn response_for_panic(
//         &mut self,
//         err: Box<dyn Any + Send + 'static>,
//     ) -> Response<Self::ResponseBody> {
//         // if let Some(s) = err.downcast_ref::<String>() {
//         //     tracing::error!("Service panicked: {}", s);
//         // } else if let Some(s) = err.downcast_ref::<&str>() {
//         //     tracing::error!("Service panicked: {}", s);
//         // } else {
//         //     tracing::error!(
//         //         "Service panicked but `CatchPanic` was unable to downcast the panic info"
//         //     );
//         // };

//         if let Some(s) = err.downcast_ref::<String>() {
//             println!("Service panicked: {}", s);
//         } else if let Some(s) = err.downcast_ref::<&str>() {
//             println!("Service panicked: {}", s);
//         } else {
//             println!(
//                 "Service panicked but `CatchPanic` was unable to downcast the panic info"
//             );
//         };

//         let rsp = AppResponse::<()> {
//             is_success: false,
//             error_code: -1,
//             error_msg: Some("服务异常".to_string()),
//             result: None,
//         };

//         let res = Response::new(rsp);
//         // *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;

//         // #[allow(clippy::declare_interior_mutable_const)]
//         // const TEXT_PLAIN: HeaderValue = HeaderValue::from_static("text/plain; charset=utf-8");
//         // res.headers_mut()
//         //     .insert(http::header::CONTENT_TYPE, TEXT_PLAIN);

//         res
//     }
// }
