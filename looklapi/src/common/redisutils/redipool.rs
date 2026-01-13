use crate::app::app_config::AppConfig;
use crate::app::appcontext;
use redis::{Client, Connection, ConnectionLike, RedisResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

const MAX_DB_INDEX: u8 = 15;

struct RedisPool {
    clients: HashMap<u8, Client>,
}

impl RedisPool {
    fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    async fn get_client(&mut self, db_index: u8) -> RedisResult<&Client> {
        if db_index > MAX_DB_INDEX {
            return Err(redis::RedisError::from((
                redis::ErrorKind::InvalidClientConfig,
                "db index out of range",
            )));
        }

        if !self.clients.contains_key(&db_index) {
            // 从AppConfig中获取redis配置
            let rudi_context = appcontext::rudi_context::RudiContext::instance();
            let context = rudi_context.read().await;
            let app_config = context.get_ctx().get_single::<AppConfig>();
            let redis_config = app_config.redis.as_ref().ok_or_else(|| {
                redis::RedisError::from((
                    redis::ErrorKind::InvalidClientConfig,
                    "redis config not found",
                ))
            })?;

            // 构建redis连接URL
            let redis_url = if redis_config.password.is_empty() {
                format!(
                    "redis://{}:{}/{}",
                    redis_config.host, redis_config.port, db_index
                )
            } else {
                format!(
                    "redis://:{}@{}:{}/{}",
                    redis_config.password, redis_config.host, redis_config.port, db_index
                )
            };

            let client = Client::open(redis_url)?;
            self.clients.insert(db_index, client);
        }

        Ok(self.clients.get(&db_index).unwrap())
    }

    async fn get_connection(&mut self, db_index: u8) -> RedisResult<Connection> {
        let client = self.get_client(db_index).await?;
        client.get_connection()
    }
}

lazy_static::lazy_static! {
    static ref REDIS_POOL: Arc<RwLock<RedisPool>> = Arc::new(RwLock::new(RedisPool::new()));
}

// 根据key获取对应的db索引
fn get_db_index_from_key(key: &str) -> u8 {
    // 这里实现与golang版本相同的逻辑，从key中提取db索引
    // 例如：5_op_shop_stock_ 表示使用db 5
    if let Some(first_char) = key.chars().next() {
        if first_char.is_digit(10) {
            let db_index = first_char.to_digit(10).unwrap() as u8;
            if db_index <= MAX_DB_INDEX {
                return db_index;
            }
        }
    }
    0 // 默认使用db 0
}

// 获取redis连接
pub async fn get_conn(key: &str) -> RedisResult<Connection> {
    let db_index = get_db_index_from_key(key);
    let mut pool = REDIS_POOL.write().await;
    pool.get_connection(db_index).await
}

// 获取指定db索引的redis连接
pub async fn get_conn0(db_index: u8) -> RedisResult<Connection> {
    let mut pool = REDIS_POOL.write().await;
    pool.get_connection(db_index).await
}
