use std::backtrace;

use mongodb::options::ConnectionString;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::app::{self, AppError};

pub async fn init_logger(cfg: &app::app_config::AppConfig) {
    match cfg.logger.default.as_str() {
        "console" => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer())
                .init();
        }
        "mongo" => {
            if cfg.mongodb.is_none() {
                panic!("mongodb config is empty");
            }
            let mongo = cfg.mongodb.as_ref().unwrap();
            let cstr = ConnectionString::parse(mongo.uri.as_str()).unwrap();
            let mongo_connection_string = mongo.uri.as_str();
            let mongo_database = cstr.default_database.unwrap();
            let mongo_collection = "system_log";

            let mongo_logger = super::mongo_logger::MongoLogger::new(
                mongo_connection_string,
                mongo_database.as_str(),
                mongo_collection,
            )
            .await
            .unwrap();

            let formatter = super::mongo_logger::MongoFormatter::new(mongo_logger);

            let fmt_layer = tracing_subscriber::fmt::layer().event_format(formatter);

            tracing_subscriber::registry().with(fmt_layer).init();
        }
        _ => {
            panic!("Invalid log type")
        }
    }
}

pub fn error(err: &AppError) {
    let backtrace = err.backtrace();
    let backtrace = format!("{}", backtrace);
    // println!("message={}, {}", err.message(), backtrace);
    tracing::error!(
        code = err.code(),
        backtrace = backtrace.as_str(),
        message = err.message()
    );
}
