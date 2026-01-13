use super::redipool::{get_conn, get_conn0};
use redis::{Commands, RedisResult};
use serde::{Serialize, de::DeserializeOwned};

// 对象转json字符串
pub fn obj_to_json<T: Serialize>(val: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string(val)
}

// json字符串转对象
pub fn json_to_obj<T: DeserializeOwned>(json: &str) -> Result<T, serde_json::Error> {
    serde_json::from_str(json)
}

// 设置键值对
pub async fn set<T: Serialize>(key: &str, val: &T) -> RedisResult<()> {
    if key.is_empty() {
        return Err(redis::RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid key",
        )));
    }

    let val_json = obj_to_json(val)?;
    let mut conn = get_conn(key).await?;
    conn.set(key, val_json)
}

// 设置带过期时间的键值对
pub async fn set_ex<T: Serialize>(key: &str, val: &T, secs: u64) -> RedisResult<()> {
    if key.is_empty() || secs == 0 {
        return Err(redis::RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid arguments",
        )));
    }

    let val_json = obj_to_json(val)?;
    let mut conn = get_conn(key).await?;
    conn.set_ex(key, val_json, secs)
}

// 获取键值
pub async fn get<T: DeserializeOwned>(key: &str) -> RedisResult<T> {
    if key.is_empty() {
        return Err(redis::RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid key",
        )));
    }

    let mut conn = get_conn(key).await?;
    let val_json: String = conn.get(key)?;
    json_to_obj(&val_json).map_err(|e| redis::RedisError::from(e))
}

// 增减值
pub async fn incr(key: &str, incr_val: i64) -> RedisResult<i64> {
    if key.is_empty() {
        return Err(redis::RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid key",
        )));
    }

    let mut conn = get_conn(key).await?;
    redis::cmd("INCRBY").arg(key).arg(incr_val).query(&mut conn)
}

// 模糊查询keys
pub async fn scan(db_index: u8, pattern: &str, limit: usize) -> RedisResult<Vec<String>> {
    if db_index > 15 {
        return Err(redis::RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "db index out of range",
        )));
    }

    if pattern.is_empty() {
        return Err(redis::RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "pattern must not be empty",
        )));
    }

    let mut conn = get_conn0(db_index).await?;
    let mut cursor = 0;
    let mut keys = Vec::new();

    loop {
        let (new_cursor, result): (u64, Vec<String>) = redis::cmd("SCAN")
            .arg(cursor)
            .arg("MATCH")
            .arg(pattern)
            .arg("COUNT")
            .arg(100)
            .query(&mut conn)?;
        keys.extend(result);

        if new_cursor == 0 || (limit > 0 && keys.len() >= limit) {
            break;
        }

        cursor = new_cursor;
    }

    if limit > 0 && keys.len() > limit {
        keys.truncate(limit);
    }

    Ok(keys)
}

// 设置哈希字段
pub async fn hash_set<T: Serialize>(key: &str, field: &str, val: &T) -> RedisResult<()> {
    if key.is_empty() || field.is_empty() {
        return Err(redis::RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid arguments",
        )));
    }

    let val_json = obj_to_json(val)?;
    let mut conn = get_conn(key).await?;
    conn.hset(key, field, val_json)
}

// 获取哈希字段
pub async fn hash_get<T: DeserializeOwned>(key: &str, field: &str) -> RedisResult<T> {
    if key.is_empty() || field.is_empty() {
        return Err(redis::RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid arguments",
        )));
    }

    let mut conn = get_conn(key).await?;
    let val_json: String = conn.hget(key, field)?;
    json_to_obj(&val_json).map_err(|e| redis::RedisError::from(e))
}

// 获取哈希所有字段
pub async fn hash_keys(key: &str) -> RedisResult<Vec<String>> {
    if key.is_empty() {
        return Err(redis::RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid key",
        )));
    }

    let mut conn = get_conn(key).await?;
    conn.hkeys(key)
}

// 获取哈希所有值
pub async fn hash_values<T: DeserializeOwned>(key: &str) -> RedisResult<Vec<T>> {
    if key.is_empty() {
        return Err(redis::RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid key",
        )));
    }

    let mut conn = get_conn(key).await?;
    let values: Vec<String> = conn.hvals(key)?;
    values
        .into_iter()
        .map(|v| json_to_obj(&v).map_err(|e| redis::RedisError::from(e)))
        .collect()
}

// 获取哈希所有字段和值
pub async fn hash_get_all<T: DeserializeOwned>(key: &str) -> RedisResult<Vec<(String, T)>> {
    if key.is_empty() {
        return Err(redis::RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid key",
        )));
    }

    let mut conn = get_conn(key).await?;
    let pairs: Vec<(String, String)> = conn.hgetall(key)?;
    pairs
        .into_iter()
        .map(|(k, v)| {
            json_to_obj(&v)
                .map(|obj| (k, obj))
                .map_err(|e| redis::RedisError::from(e))
        })
        .collect()
}

// 判断键是否存在
pub async fn exist(key: &str) -> RedisResult<bool> {
    if key.is_empty() {
        return Ok(false);
    }

    let mut conn = get_conn(key).await?;
    conn.exists(key)
}

// 判断哈希字段是否存在
pub async fn h_exist(key: &str, field: &str) -> RedisResult<bool> {
    if key.is_empty() || field.is_empty() {
        return Ok(false);
    }

    let mut conn = get_conn(key).await?;
    conn.hexists(key, field)
}

// 删除键
pub async fn del(key: &str) -> RedisResult<()> {
    if key.is_empty() {
        return Ok(());
    }

    let mut conn = get_conn(key).await?;
    let _: usize = conn.del(key)?;
    Ok(())
}

// 删除哈希字段
pub async fn h_del(key: &str, field: &str) -> RedisResult<()> {
    if key.is_empty() || field.is_empty() {
        return Ok(());
    }

    let mut conn = get_conn(key).await?;
    let _: usize = conn.hdel(key, field)?;
    Ok(())
}

// 批量删除哈希字段
pub async fn h_del_multiple(key: &str, fields: &[&str]) -> RedisResult<()> {
    if key.is_empty() || fields.is_empty() {
        return Ok(());
    }

    let mut conn = get_conn(key).await?;
    let _: usize = conn.hdel(key, fields)?;
    Ok(())
}

// 增减哈希值
pub async fn hash_incr(key: &str, field: &str, incr_val: i64) -> RedisResult<i64> {
    if key.is_empty() || field.is_empty() {
        return Err(redis::RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid arguments",
        )));
    }

    let mut conn = get_conn(key).await?;
    redis::cmd("HINCRBY")
        .arg(key)
        .arg(field)
        .arg(incr_val)
        .query(&mut conn)
}

// 获取哈希长度
pub async fn h_len(key: &str) -> RedisResult<usize> {
    if key.is_empty() {
        return Err(redis::RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid key",
        )));
    }

    let mut conn = get_conn(key).await?;
    conn.hlen(key)
}

// 设置键过期时间（秒）
pub async fn set_key_exp_secs(key: &str, exp_secs: u64) -> RedisResult<()> {
    if key.is_empty() || exp_secs == 0 {
        return Err(redis::RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid arguments",
        )));
    }

    let mut conn = get_conn(key).await?;
    conn.expire(key, exp_secs as i64)
}

// 设置键过期时间（毫秒）
pub async fn set_key_exp_millisecs(key: &str, exp_millisecs: u64) -> RedisResult<()> {
    if key.is_empty() || exp_millisecs == 0 {
        return Err(redis::RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid arguments",
        )));
    }

    let mut conn = get_conn(key).await?;
    conn.pexpire(key, exp_millisecs as i64)
}

// 设置键过期时间（时间戳，秒）
pub async fn set_key_exp_unix_secs(key: &str, exp_time: u64) -> RedisResult<()> {
    if key.is_empty() {
        return Err(redis::RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid arguments",
        )));
    }

    let mut conn = get_conn(key).await?;
    redis::cmd("EXPIREAT")
        .arg(key)
        .arg(exp_time as i64)
        .query(&mut conn)
}

// 设置键过期时间（时间戳，毫秒）
pub async fn set_key_exp_unix_millisecs(key: &str, exp_time: u64) -> RedisResult<()> {
    if key.is_empty() {
        return Err(redis::RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid arguments",
        )));
    }

    let mut conn = get_conn(key).await?;
    redis::cmd("PEXPIREAT")
        .arg(key)
        .arg(exp_time as i64)
        .query(&mut conn)
}

// 移除键过期时间
pub async fn remove_key_exp(key: &str) -> RedisResult<()> {
    if key.is_empty() {
        return Err(redis::RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid arguments",
        )));
    }

    let mut conn = get_conn(key).await?;
    conn.persist(key)
}

// 获取键剩余存活时间（秒）
pub async fn get_key_ttl_secs(key: &str) -> RedisResult<i64> {
    if key.is_empty() {
        return Err(redis::RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid arguments",
        )));
    }

    let mut conn = get_conn(key).await?;
    conn.ttl(key)
}

// 获取键剩余存活时间（毫秒）
pub async fn get_key_ttl_millisecs(key: &str) -> RedisResult<i64> {
    if key.is_empty() {
        return Err(redis::RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid arguments",
        )));
    }

    let mut conn = get_conn(key).await?;
    conn.pttl(key)
}
