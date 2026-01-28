use crate::common::mqutils::consts::ConsumerType;
use chrono::{DateTime, Utc};
use lapin::{Channel, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, AtomicI64, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex, RwLock};

/// connect 空闲超时时间 分钟
const CONN_IDLE_TIMEOUT_MIN: i32 = 10;
/// channel 空闲超时时间 分钟
const CH_IDLE_TIMEOUT_MIN: i32 = 5;

/// connect 数量限制
const CONN_LIMIT: i32 = 100;
/// channel 数量限制
const CH_LIMIT_FOR_CONN: i32 = 100;

/// 通道状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChannelStatus {
    Idle,    // 空闲
    Busy,    // 使用中
    Timeout, // 超时空闲
    Close,   // 关闭
}

/// rabbitmq连接数据
#[derive(Debug)]
pub struct RabbitMqConnData {
    /// 连接id
    pub guid: String,
    /// 连接
    pub conn: Arc<Connection>,
    /// 存活channel数
    pub live_ch: Arc<AtomicI32>,
    /// 最近一次使用时间 毫秒
    pub last_use_mills: Arc<AtomicI64>,
}

impl RabbitMqConnData {
    /// 增加channel数
    pub fn inc_chan(&self) {
        self.live_ch.fetch_add(1, Ordering::Relaxed);
        self.last_use_mills
            .store(Utc::now().timestamp_millis(), Ordering::Relaxed);
    }

    /// 减少channel数
    pub fn dec_chan(&self) {
        self.live_ch.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn new(conn: Arc<Connection>) -> Self {
        let guid = uuid::Uuid::new_v4().to_string().replace('-', "");
        Self {
            guid,
            conn,
            live_ch: Arc::new(AtomicI32::new(0)),
            last_use_mills: Arc::new(AtomicI64::new(Utc::now().timestamp_millis())),
        }
    }
}

/// MQ消息通道
#[derive(Debug)]
pub struct MqChannel {
    /// 连接信息
    pub conn: Arc<RabbitMqConnData>,
    /// 通道
    pub channel: Arc<Channel>,
    /// 通道状态
    pub status: Mutex<ChannelStatus>,
    /// 最近一次使用时间 毫秒
    pub last_use_mills: AtomicI64,
}

impl MqChannel {
    pub fn new(conn: Arc<RabbitMqConnData>, channel: Channel) -> Self {
        let now = Utc::now().timestamp_millis();
        Self {
            conn,
            channel: Arc::new(channel),
            status: Mutex::new(ChannelStatus::Busy),
            last_use_mills: AtomicI64::new(now),
        }
    }

    /// 更新最近一次使用时间
    pub fn update_last_use_mills(&self) {
        self.last_use_mills
            .store(Utc::now().timestamp_millis(), Ordering::Relaxed);
    }

    /// 设置通道状态
    pub fn set_status(&self, new_status: ChannelStatus) {
        if let Ok(mut status_guard) = self.status.lock() {
            *status_guard = new_status;
        }
    }

    /// 获取通道状态
    pub fn get_status(&self) -> ChannelStatus {
        if let Ok(status_guard) = self.status.lock() {
            *status_guard
        } else {
            ChannelStatus::Close
        }
    }
}

/// 消息体
#[derive(Debug, Serialize, Deserialize)]
pub struct MqMessage {
    /// 消息id
    pub guid: String,
    /// 消息生成时间
    pub timespan: DateTime<Utc>,
    /// 当前重试次数
    pub current_retry: i32,
    /// 消息内容
    pub json_content: String,
}

impl MqMessage {
    pub fn new() -> Self {
        Self {
            guid: uuid::Uuid::new_v4().to_string().replace('-', ""),
            timespan: Utc::now(),
            current_retry: 0,
            json_content: String::new(),
        }
    }
}

/// 消费者
pub struct Consumer {
    /// 消费者类型
    pub r#type: ConsumerType,
    /// 最大重试次数
    pub max_retry: u32,
    /// 处理器
    pub consume: Box<dyn Fn(serde_json::Value) -> bool + Send + Sync>,
    /// broadcast交换器名称
    pub exchange: String,
    /// workqueue路由地址
    pub route_key: String,
    /// workqueue并发消费者数量
    pub concurrency: u32,
    /// workqueue从队列中同时deliver的消息数量
    pub prefetch_count: u32,
    /// workqueue是否开启并行消费
    pub parallel: bool,
    /// topic模式的路由键模式
    pub topic_pattern: String,
}
