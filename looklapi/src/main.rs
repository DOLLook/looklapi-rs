use axum::{
    Extension, Router,
    body::Body,
    http::{Method, Request},
    response::IntoResponse,
    routing::*,
};
use tower_http::cors::{AllowHeaders, AllowOrigin, CorsLayer};
use tracing::info;

use crate::{
    app::{AppError, AppResponse, app_config},
    request_context::X_REQUEST_ID,
};

mod app;
mod common;
mod controller;
mod request_context;

#[tokio::main]
async fn main() {
    let mut ctx = rudi::Context::auto_register();
    let app_config = ctx.resolve::<app_config::AppConfig>();

    common::loggers::init_logger(&app_config).await;
    let app = app();

    let listen = format!("0.0.0.0:{}", app_config.server.port);
    let listener = tokio::net::TcpListener::bind(listen).await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    info!("app start 完成");
    axum::serve(listener, app).await.unwrap();
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

fn app() -> Router {
    // let request_id_middleware =
    //     ServiceBuilder::new().layer(SetRequestIdLayer::x_request_id(MakeRequestUuid));
    // send headers from request to response headers
    // .layer(PropagateRequestIdLayer::new(x_request_id));

    Router::new()
        .route(
            "/",
            get(handler)
                .layer(axum::middleware::from_fn(test_after))
                .layer(axum::middleware::from_fn(test_begin)),
        )
        .layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::any())
                .allow_headers(AllowHeaders::any())
                .allow_methods([Method::OPTIONS, Method::HEAD, Method::GET, Method::POST]),
        )
        .layer(axum::middleware::from_fn(
            controller::middleware::request_ctx_middleware,
        ))
        .layer(axum::middleware::from_fn(
            controller::middleware::request_id_middleware,
        ))
        .layer(axum::middleware::from_fn(
            controller::middleware::panic_handler,
        ))
    // .layer(request_id_middleware)
}

async fn test_begin(mut req: Request<Body>, next: axum::middleware::Next) -> impl IntoResponse {
    let req_id = req.headers().get(X_REQUEST_ID).unwrap().to_str().unwrap();

    println!("test_begin请求ID: {}", req_id);

    next.run(req).await
}

async fn test_after(mut req: Request<Body>, next: axum::middleware::Next) -> impl IntoResponse {
    let req_id = req.headers().get(X_REQUEST_ID).unwrap().to_str().unwrap();

    println!("test_after请求ID: {}", req_id);

    next.run(req).await
}
