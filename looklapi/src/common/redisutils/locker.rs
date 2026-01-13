use super::redipool::get_conn;
use redis::{Commands, RedisError, RedisResult};
use std::time::{Duration, Instant};

// 获取分布式锁
pub async fn lock(key: &str, expire_secs: u64) -> RedisResult<bool> {
    if key.is_empty() || expire_secs == 0 {
        return Err(RedisError::from((redis::ErrorKind::InvalidClientConfig, "invalid arguments")));
    }

    let mut conn = get_conn(key).await?;
    let value = format!("{}", Instant::now().duration_since(Instant::now()).as_nanos());
    
    // 使用SET命令的NX选项获取锁
    let result: Option<String> = redis::cmd("SET").arg(key).arg(&value).arg("NX").arg("EX").arg(expire_secs as i64).query(&mut conn)?;
    match result {
        Some(_) => Ok(true),
        None => Ok(false),
    }
}

// 释放分布式锁
pub async fn unlock(key: &str) -> RedisResult<()> {
    if key.is_empty() {
        return Err(RedisError::from((redis::ErrorKind::InvalidClientConfig, "invalid key")));
    }

    let mut conn = get_conn(key).await?;
    let _: usize = conn.del(key)?;
    Ok(())
}

// 尝试获取分布式锁，如果获取失败则等待重试
pub async fn try_lock(key: &str, expire_secs: u64, max_retry: usize, retry_interval_ms: u64) -> RedisResult<bool> {
    for _ in 0..max_retry {
        if lock(key, expire_secs).await? {
            return Ok(true);
        }
        // 等待一段时间后重试
        tokio::time::sleep(Duration::from_millis(retry_interval_ms)).await;
    }
    Ok(false)
}