use std::sync::Arc;
use std::sync::mpsc;
use std::thread;

use chrono::{DateTime, Local};
use mongodb::{Client, Collection};
use serde::Serialize;
use tracing::{Event, Subscriber};
use tracing_subscriber::{
    fmt::{FmtContext, FormatEvent, FormatFields, format::Writer},
    registry::LookupSpan,
};

#[derive(Serialize, Debug)]
struct SystemLog {
    /// 日志ID
    pub _id: String,
    /// 实例名
    pub instance: String,
    /// 线程id      
    pub thread_id: i32,
    /// 运行类名
    pub class_name: String,
    /// 日志等级
    pub level: i32,
    /// 宿主IP
    pub host_ip: String,
    /// 时间
    pub time: DateTime<Local>,
    /// 日志内容
    pub content: String,
    /// 堆栈信息
    pub stacktrace: String,
}

#[derive(Clone)]
pub struct MongoLogger {
    collection: Collection<SystemLog>,
    tx: mpsc::Sender<SystemLog>,
}

impl MongoLogger {
    pub async fn new(
        connection_string: &str,
        database: &str,
        collection: &str,
    ) -> Result<Self, mongodb::error::Error> {
        let client = Client::with_uri_str(connection_string).await?;
        let database = client.database(database);
        let collection = database.collection(collection);

        // 创建通道
        let (tx, rx) = mpsc::channel();

        // 创建一个新的MongoLogger实例，用于后台线程
        let bg_logger = MongoLogger {
            collection: collection.clone(),
            tx: tx.clone(),
        };

        // 启动后台线程
        thread::spawn(move || {
            // 在后台线程中创建一个Tokio运行时
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            // 在运行时中处理日志条目
            rt.block_on(async move {
                bg_logger.process_logs(rx).await;
            });
        });

        Ok(MongoLogger { collection, tx })
    }

    async fn log(&self, entry: SystemLog) -> Result<(), mongodb::error::Error> {
        self.collection.insert_one(entry).await?;
        Ok(())
    }

    // 添加处理日志的方法
    async fn process_logs(&self, rx: mpsc::Receiver<SystemLog>) {
        for entry in rx {
            if let Err(e) = self.log(entry).await {
                eprintln!("Failed to log to MongoDB: {}", e);
            }
        }
    }
}

pub struct MongoFormatter {
    mongo_logger: Arc<MongoLogger>,
}

impl MongoFormatter {
    pub fn new(mongo_logger: MongoLogger) -> Self {
        Self {
            mongo_logger: Arc::new(mongo_logger),
        }
    }
}

impl<S, N> FormatEvent<S, N> for MongoFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        // 获取时间戳
        let timestamp = Local::now();

        // 获取日志级别
        let level = *event.metadata().level();

        // 获取日志信息
        let metadata = event.metadata();
        let target = metadata.target();

        // // 格式化消息
        // let mut visitor = tracing::field::VisitFmt::new(&mut writer);
        // event.record(&mut visitor);

        let message = format!("{:?}", event);

        // // 输出到控制台
        // writeln!(
        //     writer,
        //     "{} {} {}: {}",
        //     timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
        //     level,
        //     target,
        //     // 这里需要特殊处理，因为我们已经写了消息到writer
        //     ""
        // )?;

        let lvl = match level {
            tracing::Level::TRACE => 6,
            tracing::Level::DEBUG => 5,
            tracing::Level::INFO => 4,
            tracing::Level::WARN => 3,
            tracing::Level::ERROR => 2,
        };

        let entry = SystemLog {
            _id: mongodb::bson::oid::ObjectId::new().to_hex(),
            instance: "looklapi-rs".to_string(),
            thread_id: 0,
            class_name: target.to_string(),
            level: lvl,
            host_ip: "127.0.0.1".to_string(),
            time: timestamp,
            content: message,
            stacktrace: metadata.target().to_string(),
        };

        let logger = self.mongo_logger.clone();
        // 发送日志条目到通道
        if let Err(e) = logger.tx.send(entry) {
            eprintln!("Failed to send log to background thread: {}", e);
        }

        Ok(())
    }
}
