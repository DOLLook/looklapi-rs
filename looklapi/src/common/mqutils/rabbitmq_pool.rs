use crate::app::app_config;
use crate::app::appcontext::events::AppEventBeanInjected;
use crate::app::appcontext::observer::AppObserver;
use crate::common::mqutils::models::{ChannelStatus, MqChannel, RabbitMqConnData};
use crate::{app, register_observer_for};
use chrono::Utc;
use futures::StreamExt;
use lapin::{Connection, ConnectionProperties};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::sync::{Mutex, RwLock};
use tracing;

unsafe impl Send for RabbitmqConnPool {}
unsafe impl Sync for RabbitmqConnPool {}

impl AppObserver for RabbitmqConnPool {
    fn on_application_event(&self, event: &dyn std::any::Any) {
        if event.downcast_ref::<AppEventBeanInjected>().is_some() {
            // 初始化RabbitMQ连接池
            tokio::spawn(async {
                RabbitmqConnPool::init().await;
            });
            tracing::info!("RabbitMQ connection pool initialized via AppEventBeanInjected");
        }
    }
}

// 注册RabbitmqConnPool作为应用事件观察者，订阅AppEventBeanInjected事件
register_observer_for!(RabbitmqConnPool, AppEventBeanInjected);

// 常量定义
const CONN_LIMIT: i32 = 10; // 连接池大小限制
const CH_LIMIT_FOR_CONN: i32 = 100; // 每个连接的channel限制
const CH_IDLE_TIMEOUT_MIN: i32 = 5; // channel空闲超时时间（分钟）
const CONN_IDLE_TIMEOUT_MIN: i32 = 10; // 连接空闲超时时间（分钟）

// 全局连接池实例
static GLOBAL_POOL: OnceLock<Arc<RabbitmqConnPool>> = OnceLock::new();
// static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// rabbitmq连接池
#[derive(Debug)]
pub struct RabbitmqConnPool {
    /// 连接地址（使用内部可变性）
    conn_str: Arc<Mutex<String>>,
    /// 发布连接
    pub_conns: RwLock<HashMap<String, Arc<RabbitMqConnData>>>,
    /// 发布channel pool
    pub_chs: Mutex<Vec<Arc<MqChannel>>>,
    /// 发布连接状态锁
    pub_lock: Arc<AtomicI32>,
    /// 发布channel管道
    pub_ch_pipeline: Arc<mpsc::Sender<Arc<MqChannel>>>,
    /// 发布channel管道接收端
    pub_ch_pipeline_rx: Arc<Mutex<mpsc::Receiver<Arc<MqChannel>>>>,
    /// 消费连接
    rec_conns: Mutex<Vec<Arc<RabbitMqConnData>>>,
    /// 消费者channel pool
    rec_chs: Mutex<Vec<Arc<MqChannel>>>,
    /// 消费者管理锁
    rec_mu: Arc<Mutex<()>>,
    /// 初始化标志
    initialized: AtomicBool,
}

impl RabbitmqConnPool {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(1);
        Self {
            conn_str: Arc::new(Mutex::new(String::new())),
            pub_conns: RwLock::new(HashMap::new()),
            pub_chs: Mutex::new(Vec::new()),
            pub_lock: Arc::new(AtomicI32::new(0)),
            pub_ch_pipeline: Arc::new(tx),
            pub_ch_pipeline_rx: Arc::new(Mutex::new(rx)),
            rec_conns: Mutex::new(Vec::new()),
            rec_chs: Mutex::new(Vec::new()),
            rec_mu: Arc::new(Mutex::new(())),
            initialized: AtomicBool::new(false),
        }
    }

    pub fn get_instance() -> Arc<Self> {
        GLOBAL_POOL
            .get_or_init(|| {
                // 创建一个默认实例，当真正初始化时会被替换
                Arc::new(RabbitmqConnPool::new())
            })
            .clone()
    }

    /// 初始化连接池
    pub async fn init() {
        let pool = RabbitmqConnPool::get_instance();
        if pool.initialized.swap(true, Ordering::Relaxed) {
            return; // 已经初始化过了
        }

        let rudi_context = app::appcontext::rudi_context::instance();
        let ctx = rudi_context.read().await;
        let app_config = ctx.get_ctx().get_single::<app_config::AppConfig>();
        // 修改 conn_str 字段
        if let Some(ref rabbitmq_config) = app_config.rabbitmq {
            let mut conn_str_guard = pool.conn_str.lock().await;
            *conn_str_guard = rabbitmq_config.address.clone();
        }

        // 启动发布通道管道填充协程
        let pool_clone = pool.clone();
        tokio::spawn(async move {
            loop {
                pool_clone.push_pub_ch_to_pipe().await;
                tokio::time::sleep(Duration::from_millis(100)).await; // 避免CPU占用过高
            }
        });

        // 启动定时清理任务
        let pool_clone = pool.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(60)).await; // 每分钟执行一次
                pool_clone.clear_idl_pub_conn().await;
            }
        });

        tracing::info!("RabbitMQ connection pool initialized");
    }

    // 获取发布通道
    pub async fn get_pub_channel(&self) -> Result<Arc<MqChannel>, Box<dyn std::error::Error>> {
        // 从管道中获取通道
        let mut rx = self.pub_ch_pipeline_rx.lock().await;
        match rx.try_recv() {
            Ok(ch) => {
                // 检查通道是否有效
                if ch.get_status() != ChannelStatus::Close {
                    return Ok(ch);
                }
            }
            Err(mpsc::error::TryRecvError::Empty) => {
                // tracing::error!("从管道获取发布通道失败: 通道已关闭");
            }
            Err(mpsc::error::TryRecvError::Disconnected) => {
                // tracing::error!("从管道获取发布通道失败: 通道已断开");
            }
        }

        // 如果管道中没有有效通道，创建新通道
        let conn = self.get_or_create_pub_conn().await?;
        let channel = conn.conn.create_channel().await?;
        let mq_channel = Arc::new(MqChannel::new(conn.clone(), channel));
        conn.inc_chan(); // 增加通道计数

        let mut pub_chs = self.pub_chs.lock().await;
        pub_chs.push(mq_channel.clone());
        Ok(mq_channel)
    }

    // 获取消费通道
    pub async fn get_rec_channel(&self) -> Result<Arc<MqChannel>, Box<dyn std::error::Error>> {
        // 尝试从现有通道中获取空闲通道
        let mut rec_chs = self.rec_chs.lock().await;
        for i in 0..rec_chs.len() {
            let ch = &rec_chs[i];
            if ch.get_status() == ChannelStatus::Idle {
                // 标记为使用中
                ch.set_status(ChannelStatus::Busy);
                ch.update_last_use_mills();
                return Ok(ch.clone());
            }
        }

        // 如果没有空闲通道，创建新通道
        let conn = self.get_or_create_rec_conn().await?;
        let channel = conn.conn.create_channel().await?;
        let mq_channel = Arc::new(MqChannel::new(conn.clone(), channel));
        conn.inc_chan(); // 增加通道计数

        rec_chs.push(mq_channel.clone());
        Ok(mq_channel)
    }

    // 获取或创建发布连接
    async fn get_or_create_pub_conn(
        &self,
    ) -> Result<Arc<RabbitMqConnData>, Box<dyn std::error::Error>> {
        let pub_conns = self.pub_conns.read().await;

        // 过滤出有效的连接（通道数小于限制且连接未关闭）
        let mut valid_conns = pub_conns
            .iter()
            .filter(|(_, conn)| {
                conn.live_ch.load(Ordering::Relaxed) < CH_LIMIT_FOR_CONN
                    && conn.conn.status().connected()
            })
            .map(|(_, conn)| conn.clone())
            .collect::<Vec<_>>();
        // 如果有有效连接，选择通道数最多的连接（与Go版本逻辑一致）
        if !valid_conns.is_empty() {
            valid_conns.sort_by(|a, b| {
                b.live_ch
                    .load(Ordering::Relaxed)
                    .cmp(&a.live_ch.load(Ordering::Relaxed))
            });
            return Ok(valid_conns[0].clone());
        }

        // 检查连接池是否已满
        if pub_conns.len() as i32 >= CONN_LIMIT {
            return Err("连接池已满".into());
        }
        drop(pub_conns);

        // 创建新连接
        let conn_str = {
            let conn_str_guard = self.conn_str.lock().await;
            conn_str_guard.clone()
        };
        let conn = Connection::connect(&conn_str, ConnectionProperties::default()).await?;
        let mut evnet_listener = conn.events_listener();
        tokio::spawn(async move {
            while let Some(event) = evnet_listener.next().await {
                // 监听连接事件
                tracing::info!("RabbitMQ connection event: {:?}", event);
            }
        });

        let conn_data = Arc::new(RabbitMqConnData::new(Arc::new(conn)));

        let mut pub_conns = self.pub_conns.write().await;
        pub_conns.insert(conn_data.guid.clone(), conn_data.clone());
        Ok(conn_data)
    }

    // 获取或创建消费连接
    async fn get_or_create_rec_conn(
        &self,
    ) -> Result<Arc<RabbitMqConnData>, Box<dyn std::error::Error>> {
        let mut rec_conns = self.rec_conns.lock().await;
        for conn in rec_conns.iter() {
            if conn.live_ch.load(Ordering::Relaxed) < CH_LIMIT_FOR_CONN {
                return Ok(conn.clone());
            }
        }

        // 创建新连接
        let conn_str = {
            let conn_str_guard = self.conn_str.lock().await;
            conn_str_guard.clone()
        };
        let conn = Connection::connect(&conn_str, ConnectionProperties::default()).await?;
        let conn_data = Arc::new(RabbitMqConnData::new(Arc::new(conn)));

        rec_conns.push(conn_data.clone());
        Ok(conn_data)
    }

    // 释放通道
    pub fn release_channel(&self, ch: Arc<MqChannel>) {
        ch.set_status(ChannelStatus::Idle);
        ch.update_last_use_mills();
    }

    // 向管道推送发布channel
    pub async fn push_pub_ch_to_pipe(&self) {
        // 尝试获取锁
        for _ in 0..3 {
            if self
                .pub_lock
                .compare_exchange(0, 1, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // 查找空闲通道
        let mut idle_ch: Option<Arc<MqChannel>> = None;
        let mut pub_chs = self.pub_chs.lock().await;
        for ch in pub_chs.iter() {
            let status = ch.get_status();
            if status != ChannelStatus::Busy
                && status != ChannelStatus::Close
                && ch.conn.conn.status().connected()
            {
                idle_ch = Some(ch.clone());
                break;
            }
        }

        if let Some(ch) = idle_ch {
            let prev_status = ch.get_status();

            // 标记为使用中
            ch.set_status(ChannelStatus::Busy);
            ch.update_last_use_mills();

            let ch_clone = ch.clone();

            // 释放锁
            self.pub_lock.store(0, Ordering::Relaxed);

            // 推送通道到管道
            if let Err(err) = self.pub_ch_pipeline.send(ch).await {
                tracing::error!("推送发布通道到管道失败: {:?}", err);
            } else {
                ch_clone.set_status(prev_status);
                return;
            }
        }

        // 未找到空闲通道，创建新通道
        let conn_result = self.get_or_create_pub_conn().await;
        if let Err(err) = conn_result {
            tracing::error!("获取发布连接失败: {:?}", err);
            self.pub_lock.store(0, Ordering::Relaxed);
            return;
        }

        let conn = conn_result.unwrap();
        let channel_result = conn.conn.create_channel().await;
        if let Err(err) = channel_result {
            tracing::error!("创建发布通道失败: {:?}", err);
            self.pub_lock.store(0, Ordering::Relaxed);
            return;
        }

        let channel = channel_result.unwrap();
        let mq_channel = Arc::new(MqChannel::new(conn.clone(), channel));

        pub_chs.push(mq_channel.clone());
        conn.inc_chan();

        // 释放锁
        self.pub_lock.store(0, Ordering::Relaxed);

        // 推送通道到管道
        if let Err(err) = self.pub_ch_pipeline.send(mq_channel).await {
            tracing::error!("推送发布通道到管道失败: {:?}", err);
        }
    }

    // 清理空闲发布连接
    pub async fn clear_idl_pub_conn(&self) {
        // 尝试获取锁
        let mut hold_lock = false;
        for _ in 0..3 {
            if self
                .pub_lock
                .compare_exchange(0, 1, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                hold_lock = true;
                break;
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        if !hold_lock {
            return;
        }

        // let now = Instant::now();
        let mut pub_chs = self.pub_chs.lock().await;
        let mut pub_conns = self.pub_conns.write().await;

        // 倒序遍历，方便删除
        let mut i = pub_chs.len();
        while i > 0 {
            i -= 1;
            let ch = &pub_chs[i];

            // 跳过忙碌的通道
            if ch.get_status() == ChannelStatus::Busy && ch.conn.conn.status().connected() {
                continue;
            }

            // 检查连接是否关闭
            if ch.conn.conn.status().closed() {
                // 从连接池中删除连接
                pub_conns.remove(&ch.conn.guid);
                // 从通道池中删除通道
                pub_chs.remove(i);
                continue;
            }

            // 检查是否需要清理
            let mut do_clear = false;
            if ch.get_status() == ChannelStatus::Close {
                do_clear = true;
            } else {
                // 检查空闲时间是否超过阈值
                let ch_last_use = ch.last_use_mills.load(Ordering::Relaxed);
                let now_mills = Utc::now().timestamp_millis();
                let idle_millis = now_mills - ch_last_use;
                if idle_millis >= (CH_IDLE_TIMEOUT_MIN * 60 * 1000) as i64 {
                    do_clear = true;
                    ch.set_status(ChannelStatus::Timeout);
                }
            }

            if do_clear {
                let conn = ch.conn.clone();
                let conn_guid = ch.conn.guid.clone();
                let live_ch = conn.live_ch.load(Ordering::Relaxed);

                if live_ch <= 1 {
                    // 检查连接空闲时间是否超过阈值
                    let conn_last_use = conn.last_use_mills.load(Ordering::Relaxed);
                    let now_mills = Utc::now().timestamp_millis();
                    let conn_idle_millis = now_mills - conn_last_use;
                    if conn_idle_millis >= (CONN_IDLE_TIMEOUT_MIN * 60 * 1000) as i64 {
                        let ch_clone = ch.clone();

                        // 从通道池中删除通道
                        pub_chs.remove(i);
                        // 减少通道计数
                        conn.dec_chan();
                        // 从连接池中删除连接
                        pub_conns.remove(&conn_guid);

                        if ch_clone.get_status() == ChannelStatus::Timeout {
                            let _ = ch_clone.channel.close(0, "关闭空闲通道").await;
                        }
                        if !ch_clone.conn.conn.status().closed() {
                            let _ = ch_clone.conn.conn.close(0, "关闭空闲连接").await;
                        }
                    }
                } else {
                    let ch_clone = ch.clone();
                    // 仅关闭通道
                    pub_chs.remove(i);
                    // 减少通道计数
                    conn.dec_chan();
                    if ch_clone.get_status() == ChannelStatus::Timeout
                        && !conn.conn.status().closed()
                    {
                        let _ = ch_clone.channel.close(0, "关闭空闲通道").await;
                    }
                }
            }
        }
    }
}
