use super::operator::{obj_to_json, json_to_obj};
use super::redipool::get_conn;
use redis::{Commands, RedisError, RedisResult};
use serde::{de::DeserializeOwned, Serialize};

// 向有序集合添加一个或多个成员
pub async fn zadd<T: Serialize>(key: &str, score: f64, member: &T) -> RedisResult<usize> {
    if key.is_empty() {
        return Err(RedisError::from((redis::ErrorKind::InvalidClientConfig, "invalid key")));
    }

    let member_json = obj_to_json(member)?;
    let mut conn = get_conn(key).await?;
    conn.zadd(key, member_json, score)
}

// 移除有序集合中的一个或多个成员
pub async fn zrem(key: &str, members: &[&str]) -> RedisResult<usize> {
    if key.is_empty() {
        return Err(RedisError::from((redis::ErrorKind::InvalidClientConfig, "invalid key")));
    }

    if members.is_empty() {
        return Ok(0);
    }

    let mut conn = get_conn(key).await?;
    conn.zrem(key, members)
}

// 获取有序集合中成员的分数
pub async fn zscore<T: Serialize>(key: &str, member: &T) -> RedisResult<f64> {
    if key.is_empty() {
        return Err(RedisError::from((redis::ErrorKind::InvalidClientConfig, "invalid key")));
    }

    let member_json = obj_to_json(member)?;
    let mut conn = get_conn(key).await?;
    conn.zscore(key, member_json)
}

// 增加有序集合中成员的分数
pub async fn zincrby<T: Serialize>(key: &str, increment: f64, member: &T) -> RedisResult<f64> {
    if key.is_empty() {
        return Err(RedisError::from((redis::ErrorKind::InvalidClientConfig, "invalid key")));
    }

    let member_json = obj_to_json(member)?;
    let mut conn = get_conn(key).await?;
    // 使用通用命令执行zincrby
    let result: (f64,) = redis::cmd("ZINCRBY").arg(key).arg(increment).arg(member_json).query(&mut conn)?;
    Ok(result.0)
}

// 获取有序集合的大小
pub async fn zcard(key: &str) -> RedisResult<usize> {
    if key.is_empty() {
        return Err(RedisError::from((redis::ErrorKind::InvalidClientConfig, "invalid key")));
    }

    let mut conn = get_conn(key).await?;
    conn.zcard(key)
}

// 获取有序集合中指定分数范围的成员
pub async fn zrangebyscore<T: DeserializeOwned>(key: &str, min: f64, max: f64) -> RedisResult<Vec<T>> {
    if key.is_empty() {
        return Err(RedisError::from((redis::ErrorKind::InvalidClientConfig, "invalid key")));
    }

    let mut conn = get_conn(key).await?;
    let members: Vec<String> = conn.zrangebyscore(key, min, max)?;
    members.into_iter().map(|m| json_to_obj(&m).map_err(|e| RedisError::from(e))).collect()
}

// 获取有序集合中指定排名范围的成员（从小到大）
pub async fn zrange<T: DeserializeOwned>(key: &str, start: isize, stop: isize) -> RedisResult<Vec<T>> {
    if key.is_empty() {
        return Err(RedisError::from((redis::ErrorKind::InvalidClientConfig, "invalid key")));
    }

    let mut conn = get_conn(key).await?;
    let members: Vec<String> = conn.zrange(key, start, stop)?;
    members.into_iter().map(|m| json_to_obj(&m).map_err(|e| RedisError::from(e))).collect()
}

// 获取有序集合中指定排名范围的成员（从大到小）
pub async fn zrevrange<T: DeserializeOwned>(key: &str, start: isize, stop: isize) -> RedisResult<Vec<T>> {
    if key.is_empty() {
        return Err(RedisError::from((redis::ErrorKind::InvalidClientConfig, "invalid key")));
    }

    let mut conn = get_conn(key).await?;
    let members: Vec<String> = conn.zrevrange(key, start, stop)?;
    members.into_iter().map(|m| json_to_obj(&m).map_err(|e| RedisError::from(e))).collect()
}

// 获取有序集合中成员的排名（从小到大，从0开始）
pub async fn zrank<T: Serialize>(key: &str, member: &T) -> RedisResult<Option<usize>> {
    if key.is_empty() {
        return Err(RedisError::from((redis::ErrorKind::InvalidClientConfig, "invalid key")));
    }

    let member_json = obj_to_json(member)?;
    let mut conn = get_conn(key).await?;
    conn.zrank(key, member_json)
}

// 获取有序集合中成员的排名（从大到小，从0开始）
pub async fn zrevrank<T: Serialize>(key: &str, member: &T) -> RedisResult<Option<usize>> {
    if key.is_empty() {
        return Err(RedisError::from((redis::ErrorKind::InvalidClientConfig, "invalid key")));
    }

    let member_json = obj_to_json(member)?;
    let mut conn = get_conn(key).await?;
    conn.zrevrank(key, member_json)
}

// 移除有序集合中指定分数范围的成员
pub async fn zremrangebyscore(key: &str, min: f64, max: f64) -> RedisResult<usize> {
    if key.is_empty() {
        return Err(RedisError::from((redis::ErrorKind::InvalidClientConfig, "invalid key")));
    }

    let mut conn = get_conn(key).await?;
    // 使用通用命令执行zremrangebyscore
    let result: (usize,) = redis::cmd("ZREMRANGEBYSCORE").arg(key).arg(min).arg(max).query(&mut conn)?;
    Ok(result.0)
}

// 移除有序集合中指定排名范围的成员
pub async fn zremrangebyrank(key: &str, start: isize, stop: isize) -> RedisResult<usize> {
    if key.is_empty() {
        return Err(RedisError::from((redis::ErrorKind::InvalidClientConfig, "invalid key")));
    }

    let mut conn = get_conn(key).await?;
    conn.zremrangebyrank(key, start, stop)
}