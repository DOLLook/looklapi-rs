use super::operator::{json_to_obj, obj_to_json};
use super::redipool::get_conn;
use redis::{Commands, RedisError, RedisResult};
use serde::{Serialize, de::DeserializeOwned};

// 向集合添加一个或多个成员
pub async fn sadd<T: Serialize>(key: &str, members: &[T]) -> RedisResult<usize> {
    if key.is_empty() {
        return Err(RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid key",
        )));
    }

    if members.is_empty() {
        return Ok(0);
    }

    let mut conn = get_conn(key).await?;
    let mut pipe = redis::pipe();

    for member in members {
        let member_json = obj_to_json(member)?;
        pipe.sadd(key, member_json);
    }

    let results: Vec<usize> = pipe.query(&mut conn)?;
    Ok(results.iter().sum())
}

// 移除集合中的一个或多个成员
pub async fn srem(key: &str, members: &[&str]) -> RedisResult<usize> {
    if key.is_empty() {
        return Err(RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid key",
        )));
    }

    if members.is_empty() {
        return Ok(0);
    }

    let mut conn = get_conn(key).await?;
    conn.srem(key, members)
}

// 判断成员是否在集合中
pub async fn sismember<T: Serialize>(key: &str, member: &T) -> RedisResult<bool> {
    if key.is_empty() {
        return Err(RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid key",
        )));
    }

    let member_json = obj_to_json(member)?;
    let mut conn = get_conn(key).await?;
    conn.sismember(key, member_json)
}

// 获取集合中的所有成员
pub async fn smembers<T: DeserializeOwned>(key: &str) -> RedisResult<Vec<T>> {
    if key.is_empty() {
        return Err(RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid key",
        )));
    }

    let mut conn = get_conn(key).await?;
    let members: Vec<String> = conn.smembers(key)?;
    members
        .into_iter()
        .map(|m| json_to_obj(&m).map_err(|e| RedisError::from(e)))
        .collect()
}

// 获取集合的大小
pub async fn scard(key: &str) -> RedisResult<usize> {
    if key.is_empty() {
        return Err(RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid key",
        )));
    }

    let mut conn = get_conn(key).await?;
    conn.scard(key)
}

// 从集合中随机移除并返回一个成员
pub async fn spop<T: DeserializeOwned>(key: &str) -> RedisResult<T> {
    if key.is_empty() {
        return Err(RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid key",
        )));
    }

    let mut conn = get_conn(key).await?;
    let member_json: String = conn.spop(key)?;
    json_to_obj(&member_json).map_err(|e| RedisError::from(e))
}

// 从集合中随机返回指定数量的成员
pub async fn srandmember<T: DeserializeOwned>(key: &str, count: usize) -> RedisResult<Vec<T>> {
    if key.is_empty() {
        return Err(RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid key",
        )));
    }

    let mut conn = get_conn(key).await?;
    let mut members = Vec::new();
    for _ in 0..count {
        if let Ok(member_json) = conn.srandmember::<_, String>(key) {
            members.push(member_json);
        }
    }
    members
        .into_iter()
        .map(|m| json_to_obj(&m).map_err(|e| RedisError::from(e)))
        .collect()
}

// 计算两个集合的差集
pub async fn sdiff<T: DeserializeOwned>(key1: &str, key2: &str) -> RedisResult<Vec<T>> {
    if key1.is_empty() || key2.is_empty() {
        return Err(RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid key",
        )));
    }

    let mut conn = get_conn(key1).await?;
    let members: Vec<String> = conn.sdiff(vec![key1, key2])?;
    members
        .into_iter()
        .map(|m| json_to_obj(&m).map_err(|e| RedisError::from(e)))
        .collect()
}

// 计算两个集合的交集
pub async fn sinter<T: DeserializeOwned>(key1: &str, key2: &str) -> RedisResult<Vec<T>> {
    if key1.is_empty() || key2.is_empty() {
        return Err(RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid key",
        )));
    }

    let mut conn = get_conn(key1).await?;
    let members: Vec<String> = conn.sinter(vec![key1, key2])?;
    members
        .into_iter()
        .map(|m| json_to_obj(&m).map_err(|e| RedisError::from(e)))
        .collect()
}

// 计算两个集合的并集
pub async fn sunion<T: DeserializeOwned>(key1: &str, key2: &str) -> RedisResult<Vec<T>> {
    if key1.is_empty() || key2.is_empty() {
        return Err(RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid key",
        )));
    }

    let mut conn = get_conn(key1).await?;
    let members: Vec<String> = conn.sunion(vec![key1, key2])?;
    members
        .into_iter()
        .map(|m| json_to_obj(&m).map_err(|e| RedisError::from(e)))
        .collect()
}
