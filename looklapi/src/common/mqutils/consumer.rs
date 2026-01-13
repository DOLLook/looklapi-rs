use crate::common::mqutils::consts::ConsumerType;
use crate::common::mqutils::models::MqMessage;
use crate::common::mqutils::rabbitmq_pool::RabbitmqConnPool;
use futures::StreamExt;
use lapin;
use lazy_static::lazy_static;
use serde_json;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tokio;
use tracing;

lazy_static! {
    static ref CONSUMER_CONTAINER: Arc<Mutex<Vec<Arc<Consumer>>>> =
        Arc::new(Mutex::new(Vec::new()));
    static ref HAS_CONSUMER_BIND: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

// 消费者
pub struct Consumer {
    pub r#type: ConsumerType,
    pub max_retry: u32,
    pub consume: Arc<dyn Fn(serde_json::Value) -> bool + Send + Sync>,
    pub exchange: String,
    pub route_key: String,
    pub concurrency: u32,
    pub prefetch_count: u32,
    pub parallel: bool,
    pub topic_pattern: String,
}

impl Consumer {
    // 新建工作队列消费者
    pub fn new_work_queue_consumer(
        route_key: &str,
        concurrency: u32,
        prefetch_count: u32,
        parallel: bool,
        max_retry: u32,
        consume: impl Fn(serde_json::Value) -> bool + Send + Sync + 'static,
    ) -> Arc<Self> {
        if route_key.is_empty() {
            panic!("invalid routekey");
        }

        if concurrency < 1 {
            panic!("workqueue consumer concurrency must greater than 0");
        }

        if prefetch_count < 1 {
            panic!("workqueue consumer prefetchCount must greater than 0");
        }

        if max_retry < 1 {
            panic!("workqueue consumer maxRetry must greater than 0");
        }

        let consumer = Arc::new(Self {
            r#type: ConsumerType::WorkQueue,
            max_retry,
            consume: Arc::new(consume),
            exchange: String::new(),
            route_key: route_key.to_string(),
            concurrency,
            prefetch_count,
            parallel,
            topic_pattern: String::new(),
        });

        let mut container = CONSUMER_CONTAINER.lock().unwrap();
        container.push(consumer.clone());

        consumer
    }

    // 新建广播消费者
    pub fn new_broadcast_consumer(
        exchange: &str,
        max_retry: u32,
        consume: impl Fn(serde_json::Value) -> bool + Send + Sync + 'static,
    ) -> Arc<Self> {
        if exchange.is_empty() {
            panic!("invalid exchange");
        }

        if max_retry < 1 {
            panic!("broadcast consumer maxRetry must greater than 0");
        }

        let consumer = Arc::new(Self {
            r#type: ConsumerType::Broadcast,
            max_retry,
            consume: Arc::new(consume),
            exchange: exchange.to_string(),
            route_key: String::new(),
            concurrency: 1,
            prefetch_count: 1,
            parallel: false,
            topic_pattern: String::new(),
        });

        let mut container = CONSUMER_CONTAINER.lock().unwrap();
        container.push(consumer.clone());

        consumer
    }

    // 新建topic消费者
    pub fn new_topic_consumer(
        exchange: &str,
        topic_pattern: &str,
        max_retry: u32,
        consume: impl Fn(serde_json::Value) -> bool + Send + Sync + 'static,
    ) -> Arc<Self> {
        if exchange.is_empty() {
            panic!("invalid exchange");
        }

        if topic_pattern.is_empty() {
            panic!("invalid topic pattern");
        }

        if max_retry < 1 {
            panic!("topic consumer maxRetry must greater than 0");
        }

        let consumer = Arc::new(Self {
            r#type: ConsumerType::Topic,
            max_retry,
            consume: Arc::new(consume),
            exchange: exchange.to_string(),
            route_key: String::new(),
            concurrency: 1,
            prefetch_count: 1,
            parallel: false,
            topic_pattern: topic_pattern.to_string(),
        });

        let mut container = CONSUMER_CONTAINER.lock().unwrap();
        container.push(consumer.clone());

        consumer
    }

    // 接收到消息
    pub fn on_received(&self, msg: &str) -> bool {
        if msg.is_empty() {
            return true;
        }

        let meta_msg: MqMessage = match serde_json::from_str(msg) {
            Ok(msg) => msg,
            Err(err) => {
                tracing::error!("消息反序列化失败: {:?}", err);
                return true;
            }
        };

        if meta_msg.json_content.is_empty() {
            return true;
        }

        let msg_value: serde_json::Value = match serde_json::from_str(&meta_msg.json_content) {
            Ok(value) => value,
            Err(err) => {
                tracing::error!("消息内容反序列化失败: {:?}", err);
                return false;
            }
        };

        (self.consume)(msg_value)
    }
}

// 处理工作队列重连
async fn handle_workqueue_reconnect(
    rx: mpsc::Receiver<Arc<Consumer>>,
    tx: Option<mpsc::Sender<Arc<Consumer>>>,
) {
    let mut last_reconnect_ok = true;
    let mut continue_err = 0;

    for consumer in rx {
        if !last_reconnect_ok {
            if continue_err > 10 {
                continue_err = 10;
            }
            let wait_secs = continue_err;
            let wait_secs = if wait_secs < 1 { 1 } else { wait_secs };
            tracing::warn!(
                "last reconnect failed, wait {} seconds to reconnect",
                wait_secs
            );
            tokio::time::sleep(Duration::from_secs(wait_secs as u64)).await;
        }

        let binder = ConsumerBinder::new();
        last_reconnect_ok = binder.bind_work_queue_consumer(consumer.clone()).await;
        if last_reconnect_ok {
            continue_err = 0;
            tracing::warn!(
                "reconnect success, workqueue consumer: {}",
                consumer.route_key
            );
        } else {
            continue_err += 1;
            if let Some(ref ch) = tx {
                ch.send(consumer).unwrap();
            }
        }
    }
}

// 处理广播重连
async fn handle_broadcast_reconnect(
    rx: mpsc::Receiver<Arc<Consumer>>,
    tx: Option<mpsc::Sender<Arc<Consumer>>>,
) {
    let mut last_reconnect_ok = true;
    let mut continue_err = 0;

    for consumer in rx {
        if !last_reconnect_ok {
            if continue_err > 10 {
                continue_err = 10;
            }
            let wait_secs = continue_err;
            let wait_secs = if wait_secs < 1 { 1 } else { wait_secs };
            tracing::warn!(
                "last reconnect failed, wait {} seconds to reconnect",
                wait_secs
            );
            tokio::time::sleep(Duration::from_secs(wait_secs as u64)).await;
        }

        let binder = ConsumerBinder::new();
        last_reconnect_ok = binder.bind_broadcast_consumer(consumer.clone()).await;
        if last_reconnect_ok {
            continue_err = 0;
            tracing::warn!(
                "reconnect success, broadcast consumer: {}",
                consumer.exchange
            );
        } else {
            continue_err += 1;
            if let Some(ref ch) = tx {
                ch.send(consumer).unwrap();
            }
        }
    }
}

// 处理topic重连
async fn handle_topic_reconnect(
    rx: mpsc::Receiver<Arc<Consumer>>,
    tx: Option<mpsc::Sender<Arc<Consumer>>>,
) {
    let mut last_reconnect_ok = true;
    let mut continue_err = 0;

    for consumer in rx {
        if !last_reconnect_ok {
            if continue_err > 10 {
                continue_err = 10;
            }
            let wait_secs = continue_err;
            let wait_secs = if wait_secs < 1 { 1 } else { wait_secs };
            tracing::warn!(
                "last reconnect failed, wait {} seconds to reconnect",
                wait_secs
            );
            tokio::time::sleep(Duration::from_secs(wait_secs as u64)).await;
        }

        let binder = ConsumerBinder::new();
        last_reconnect_ok = binder.bind_topic_consumer(consumer.clone()).await;
        if last_reconnect_ok {
            continue_err = 0;
            tracing::warn!(
                "reconnect success, topic consumer: {} - {}",
                consumer.exchange,
                consumer.topic_pattern
            );
        } else {
            continue_err += 1;
            if let Some(ref ch) = tx {
                ch.send(consumer).unwrap();
            }
        }
    }
}

// 消费者绑定器
struct ConsumerBinder {
    workqueue_reconnect_ch: Option<mpsc::Sender<Arc<Consumer>>>,
    broadcast_reconnect_ch: Option<mpsc::Sender<Arc<Consumer>>>,
    topic_reconnect_ch: Option<mpsc::Sender<Arc<Consumer>>>,
}

impl ConsumerBinder {
    fn new() -> Self {
        Self {
            workqueue_reconnect_ch: None,
            broadcast_reconnect_ch: None,
            topic_reconnect_ch: None,
        }
    }

    // 绑定消费者
    pub async fn bind_consumer(&self, consumer: Arc<Consumer>) {
        match consumer.r#type {
            ConsumerType::WorkQueue => {
                for _ in 0..consumer.concurrency {
                    if !self.bind_work_queue_consumer(consumer.clone()).await {
                        if let Some(ch) = &self.workqueue_reconnect_ch {
                            ch.send(consumer.clone()).unwrap();
                        }
                    }
                }
            }
            ConsumerType::Broadcast => {
                if !self.bind_broadcast_consumer(consumer.clone()).await {
                    if let Some(ch) = &self.broadcast_reconnect_ch {
                        ch.send(consumer.clone()).unwrap();
                    }
                }
            }
            ConsumerType::Topic => {
                if !self.bind_topic_consumer(consumer.clone()).await {
                    if let Some(ch) = &self.topic_reconnect_ch {
                        ch.send(consumer.clone()).unwrap();
                    }
                }
            }
            _ => {
                tracing::error!("invalid consumer type");
            }
        }
    }

    // 绑定工作队列消费者
    async fn bind_work_queue_consumer(&self, consumer: Arc<Consumer>) -> bool {
        let pool = RabbitmqConnPool::get_instance();
        let rec_chan = match pool.get_rec_channel().await {
            Ok(chan) => chan,
            Err(err) => {
                tracing::error!("获取消费通道失败: {:?}", err);
                return false;
            }
        };

        // 声明队列
        if let Err(err) = rec_chan
            .channel
            .queue_declare(
                &consumer.route_key,
                lapin::options::QueueDeclareOptions {
                    durable: true,
                    auto_delete: false,
                    exclusive: false,
                    ..Default::default()
                },
                lapin::types::FieldTable::default(),
            )
            .await
        {
            tracing::error!("声明队列失败: {:?}", err);
            return false;
        }

        // 设置QoS
        if let Err(err) = rec_chan
            .channel
            .basic_qos(
                consumer.prefetch_count as u16,
                lapin::options::BasicQosOptions::default(),
            )
            .await
        {
            tracing::error!("设置QoS失败: {:?}", err);
            return false;
        }

        // 消费消息
        let consumer_tag = format!("workqueue_{}", consumer.route_key);
        let consumer_clone = consumer.clone();
        let pool_clone = pool.clone();

        let _ = rec_chan
            .channel
            .clone()
            .basic_consume(
                &consumer.route_key,
                &consumer_tag,
                lapin::options::BasicConsumeOptions {
                    no_ack: false,
                    exclusive: false,
                    no_local: false,
                    ..Default::default()
                },
                lapin::types::FieldTable::default(),
            )
            .await
            .map(|consumer| {
                tokio::spawn(async move {
                    let mut consumer = consumer;
                    while let Some(delivery) = consumer.next().await {
                        match delivery {
                            Ok(delivery) => {
                                let content = String::from_utf8_lossy(&delivery.data).to_string();
                                let result = consumer_clone.on_received(&content);

                                if result {
                                    let _ = delivery
                                        .ack(lapin::options::BasicAckOptions::default())
                                        .await;
                                } else {
                                    let _ = delivery
                                        .nack(lapin::options::BasicNackOptions {
                                            requeue: true,
                                            ..Default::default()
                                        })
                                        .await;
                                }
                            }
                            Err(err) => {
                                tracing::error!("消费消息失败: {:?}", err);
                                break;
                            }
                        }
                    }
                });
            });

        true
    }

    // 绑定广播消费者
    async fn bind_broadcast_consumer(&self, consumer: Arc<Consumer>) -> bool {
        let pool = RabbitmqConnPool::get_instance();
        let rec_chan = match pool.get_rec_channel().await {
            Ok(chan) => chan,
            Err(err) => {
                tracing::error!("获取消费通道失败: {:?}", err);
                return false;
            }
        };

        // 声明交换器
        if let Err(err) = rec_chan
            .channel
            .exchange_declare(
                &consumer.exchange,
                lapin::ExchangeKind::Fanout,
                lapin::options::ExchangeDeclareOptions {
                    durable: false,
                    auto_delete: true,
                    internal: false,
                    ..Default::default()
                },
                lapin::types::FieldTable::default(),
            )
            .await
        {
            tracing::error!("声明交换器失败: {:?}", err);
            return false;
        }

        // 声明临时队列
        let queue = match rec_chan
            .channel
            .queue_declare(
                "", // name
                lapin::options::QueueDeclareOptions {
                    durable: false,
                    auto_delete: true,
                    exclusive: true,
                    ..Default::default()
                },
                lapin::types::FieldTable::default(),
            )
            .await
        {
            Ok(queue) => queue,
            Err(err) => {
                tracing::error!("声明队列失败: {:?}", err);
                return false;
            }
        };

        // 绑定队列到交换器
        if let Err(err) = rec_chan
            .channel
            .queue_bind(
                queue.name().as_str(),
                &consumer.exchange,
                "", // routing_key
                lapin::options::QueueBindOptions::default(),
                lapin::types::FieldTable::default(),
            )
            .await
        {
            tracing::error!("绑定队列失败: {:?}", err);
            return false;
        }

        // 消费消息
        let consumer_tag = format!("broadcast_{}", consumer.exchange);
        let consumer_clone = consumer.clone();

        let _ = rec_chan
            .channel
            .clone()
            .basic_consume(
                queue.name().as_str(),
                &consumer_tag,
                lapin::options::BasicConsumeOptions {
                    no_ack: false,
                    exclusive: false,
                    no_local: false,
                    ..Default::default()
                },
                lapin::types::FieldTable::default(),
            )
            .await
            .map(|consumer| {
                tokio::spawn(async move {
                    let mut consumer = consumer;
                    while let Some(delivery) = consumer.next().await {
                        match delivery {
                            Ok(delivery) => {
                                let content = String::from_utf8_lossy(&delivery.data).to_string();
                                let result = consumer_clone.on_received(&content);

                                if result {
                                    let _ = delivery
                                        .ack(lapin::options::BasicAckOptions::default())
                                        .await;
                                } else {
                                    let _ = delivery
                                        .nack(lapin::options::BasicNackOptions {
                                            requeue: true,
                                            ..Default::default()
                                        })
                                        .await;
                                }
                            }
                            Err(err) => {
                                tracing::error!("消费消息失败: {:?}", err);
                                break;
                            }
                        }
                    }
                });
            });

        true
    }

    // 绑定topic消费者
    async fn bind_topic_consumer(&self, consumer: Arc<Consumer>) -> bool {
        let pool = RabbitmqConnPool::get_instance();
        let rec_chan = match pool.get_rec_channel().await {
            Ok(chan) => chan,
            Err(err) => {
                tracing::error!("获取消费通道失败: {:?}", err);
                return false;
            }
        };

        // 声明交换器
        if let Err(err) = rec_chan
            .channel
            .exchange_declare(
                &consumer.exchange,
                lapin::ExchangeKind::Topic,
                lapin::options::ExchangeDeclareOptions {
                    durable: true,
                    auto_delete: false,
                    internal: false,
                    ..Default::default()
                },
                lapin::types::FieldTable::default(),
            )
            .await
        {
            tracing::error!("声明交换器失败: {:?}", err);
            return false;
        }

        // 声明队列
        let queue_name = format!("topic_{}_{}", consumer.exchange, consumer.topic_pattern);
        let queue = match rec_chan
            .channel
            .queue_declare(
                &queue_name,
                lapin::options::QueueDeclareOptions {
                    durable: true,
                    auto_delete: false,
                    exclusive: false,
                    ..Default::default()
                },
                lapin::types::FieldTable::default(),
            )
            .await
        {
            Ok(queue) => queue,
            Err(err) => {
                tracing::error!("声明队列失败: {:?}", err);
                return false;
            }
        };

        // 绑定队列到交换器
        if let Err(err) = rec_chan
            .channel
            .queue_bind(
                queue.name().as_str(),
                &consumer.exchange,
                &consumer.topic_pattern,
                lapin::options::QueueBindOptions::default(),
                lapin::types::FieldTable::default(),
            )
            .await
        {
            tracing::error!("绑定队列失败: {:?}", err);
            return false;
        }

        // 设置QoS
        if let Err(err) = rec_chan
            .channel
            .basic_qos(
                1, // prefetch_count
                lapin::options::BasicQosOptions::default(),
            )
            .await
        {
            tracing::error!("设置QoS失败: {:?}", err);
            return false;
        }

        // 消费消息
        let consumer_tag = format!("topic_{}_{}", consumer.exchange, consumer.topic_pattern);
        let consumer_clone = consumer.clone();

        let _ = rec_chan
            .channel
            .clone()
            .basic_consume(
                queue.name().as_str(),
                &consumer_tag,
                lapin::options::BasicConsumeOptions {
                    no_ack: false,
                    exclusive: false,
                    no_local: false,
                    ..Default::default()
                },
                lapin::types::FieldTable::default(),
            )
            .await
            .map(|consumer| {
                tokio::spawn(async move {
                    let mut consumer = consumer;
                    while let Some(delivery) = consumer.next().await {
                        match delivery {
                            Ok(delivery) => {
                                let content = String::from_utf8_lossy(&delivery.data).to_string();
                                let result = consumer_clone.on_received(&content);

                                if result {
                                    let _ = delivery
                                        .ack(lapin::options::BasicAckOptions::default())
                                        .await;
                                } else {
                                    let _ = delivery
                                        .nack(lapin::options::BasicNackOptions {
                                            requeue: true,
                                            ..Default::default()
                                        })
                                        .await;
                                }
                            }
                            Err(err) => {
                                tracing::error!("消费消息失败: {:?}", err);
                                break;
                            }
                        }
                    }
                });
            });

        true
    }

    // 初始化消费者
    pub async fn init_consumers(&mut self) {
        if HAS_CONSUMER_BIND.load(std::sync::atomic::Ordering::SeqCst) {
            return;
        }

        let container = CONSUMER_CONTAINER.lock().unwrap();
        let mut worker_count = 0;
        let mut broadcaster_count = 0;
        let mut topic_count = 0;

        for consumer in container.iter() {
            match consumer.r#type {
                ConsumerType::WorkQueue => {
                    worker_count += consumer.concurrency as usize;
                }
                ConsumerType::Broadcast => {
                    broadcaster_count += 1;
                }
                ConsumerType::Topic => {
                    topic_count += 1;
                }
                _ => {}
            }
        }

        // 创建重连通道
        let (work_tx, work_rx) = mpsc::channel();
        let (broadcast_tx, broadcast_rx) = mpsc::channel();
        let (topic_tx, topic_rx) = mpsc::channel();

        self.workqueue_reconnect_ch = Some(work_tx);
        self.broadcast_reconnect_ch = Some(broadcast_tx);
        self.topic_reconnect_ch = Some(topic_tx);

        // 启动工作队列重连线程
        let work_tx = self.workqueue_reconnect_ch.clone();
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                handle_workqueue_reconnect(work_rx, work_tx).await;
            });
        });

        // 启动广播重连线程
        let broadcast_tx = self.broadcast_reconnect_ch.clone();
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                handle_broadcast_reconnect(broadcast_rx, broadcast_tx).await;
            });
        });

        // 启动topic重连线程
        let topic_tx = self.topic_reconnect_ch.clone();
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                handle_topic_reconnect(topic_rx, topic_tx).await;
            });
        });

        // 绑定所有消费者
        for consumer in container.iter() {
            self.bind_consumer(consumer.clone()).await;
        }

        HAS_CONSUMER_BIND.store(true, std::sync::atomic::Ordering::SeqCst);
        tracing::info!("mq init complete");
    }
}
