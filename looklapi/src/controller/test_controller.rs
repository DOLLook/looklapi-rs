use axum::{Extension, Router, body::Body, http::Request, response::IntoResponse, routing::get};
use inventory::submit;

use crate::{
    app::{AppError, AppResponse},
    controller::Controller,
    request_context::{self, X_REQUEST_ID},
};

/// 测试控制器
pub struct TestController;

impl Controller for TestController {
    fn routes() -> Router {
        Router::new()
            .route("/test", get(test_handler))
            .route("/test/hello", get(hello_handler))
            .route(
                "/test/handler",
                get(handler)
                    .layer(axum::middleware::from_fn(test_after))
                    .layer(axum::middleware::from_fn(test_begin)),
            )
    }
}

// 注册控制器
// submit! {
//     crate::controller::registry::ControllerRegistration {
//         routes: TestController::routes,
//     }
// }
crate::register_controller!(TestController);

async fn test_handler(
    Extension(ctx): Extension<crate::request_context::RequestContext>,
) -> Result<AppResponse<&'static str>, AppError> {
    println!("Test controller handler called");
    Ok(AppResponse::new("Test endpoint"))
}

async fn hello_handler() -> Result<AppResponse<&'static str>, AppError> {
    Ok(AppResponse::new("Hello from test controller"))
}

trait TestTrait {
    fn proxy_fn(&self) -> Result<(), AppError>;
    fn other_fn(&self);
}

struct Test;

impl TestTrait for Test {
    fn proxy_fn(&self) -> Result<(), AppError> {
        Err(AppError::new("test error"))
    }

    fn other_fn(&self) {
        todo!()
    }
}

// #[proxy(TestTrait)]
struct TestProxy {
    inner: Test,
}

impl TestProxy {
    fn proxy_fn(&self) -> Result<(), AppError> {
        println!("begin test");
        let result = self.inner.proxy_fn();
        println!("after test");
        result
    }

    fn other_fn(&self) {
        self.inner.other_fn();
    }
}

async fn handler(
    Extension(ctx): Extension<request_context::RequestContext>,
) -> Result<AppResponse<i32>, AppError> {
    for (k, v) in ctx.header.iter() {
        println!("{}: {}", k, v.to_str().unwrap_or("few"));
    }
    // panic!("panic123455");
    try_thing()?;
    Ok(AppResponse::new(123))
}

fn try_thing() -> Result<(), anyhow::Error> {
    anyhow::bail!("it failed!")
}

async fn test_begin(mut req: Request<Body>, next: axum::middleware::Next) -> impl IntoResponse {
    let req_id = req.headers().get(X_REQUEST_ID).unwrap().to_str().unwrap();

    println!("test_begin请求ID: {}", req_id);

    next.run(req).await
}

async fn test_after(mut req: Request<Body>, next: axum::middleware::Next) -> impl IntoResponse {
    let req_id = req.headers().get(X_REQUEST_ID).unwrap().to_str().unwrap().to_string();

    let rsp = next.run(req).await;
    println!("test_after请求ID: {}", req_id);
    rsp
}
