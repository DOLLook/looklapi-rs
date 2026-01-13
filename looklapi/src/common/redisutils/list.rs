use super::operator::{json_to_obj, obj_to_json};
use super::redipool::get_conn;
use redis::{Commands, RedisError, RedisResult};
use serde::{Serialize, de::DeserializeOwned};

// 向列表头(左端)push数据
pub async fn lpush<T: Serialize>(key: &str, values: &[T]) -> RedisResult<usize> {
    if key.is_empty() {
        return Err(RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid key",
        )));
    }

    if values.is_empty() {
        return Ok(0);
    }

    let mut conn = get_conn(key).await?;
    let mut pipe = redis::pipe();

    for val in values {
        let val_json = obj_to_json(val)?;
        pipe.lpush(key, val_json);
    }

    let results: Vec<usize> = pipe.query(&mut conn)?;
    results
        .last()
        .copied()
        .ok_or_else(|| RedisError::from((redis::ErrorKind::InvalidClientConfig, "lpush failed")))
}

// 向列表尾(右端)push数据
pub async fn rpush<T: Serialize>(key: &str, values: &[T]) -> RedisResult<usize> {
    if key.is_empty() {
        return Err(RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid key",
        )));
    }

    if values.is_empty() {
        return Ok(0);
    }

    let mut conn = get_conn(key).await?;
    let mut pipe = redis::pipe();

    for val in values {
        let val_json = obj_to_json(val)?;
        pipe.rpush(key, val_json);
    }

    let results: Vec<usize> = pipe.query(&mut conn)?;
    results
        .last()
        .copied()
        .ok_or_else(|| RedisError::from((redis::ErrorKind::InvalidClientConfig, "rpush failed")))
}

// 移除并返回表头(左端)数据
pub async fn lpop<T: DeserializeOwned>(key: &str) -> RedisResult<T> {
    if key.is_empty() {
        return Err(RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid key",
        )));
    }

    let mut conn = get_conn(key).await?;
    let val_json: String = conn.lpop(key, None)?;
    json_to_obj(&val_json).map_err(|e| RedisError::from(e))
}

// 移除并返回表尾(右端)数据
pub async fn rpop<T: DeserializeOwned>(key: &str) -> RedisResult<T> {
    if key.is_empty() {
        return Err(RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid key",
        )));
    }

    let mut conn = get_conn(key).await?;
    let val_json: String = conn.rpop(key, None)?;
    json_to_obj(&val_json).map_err(|e| RedisError::from(e))
}

// 从一个列表尾弹出数据并push到另一个列表头
pub async fn rpoplpush<T: DeserializeOwned>(
    source_key: &str,
    destination_key: &str,
) -> RedisResult<T> {
    if source_key.is_empty() || destination_key.is_empty() {
        return Err(RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid key",
        )));
    }

    let mut conn = get_conn(source_key).await?;
    let val_json: String = conn.rpoplpush(source_key, destination_key)?;
    json_to_obj(&val_json).map_err(|e| RedisError::from(e))
}

// 移除列表中与value相等的元素
pub async fn lremove(key: &str, count: isize, value: &str) -> RedisResult<usize> {
    if key.is_empty() {
        return Err(RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid key",
        )));
    }

    let mut conn = get_conn(key).await?;
    conn.lrem(key, count, value)
}

// 获取列表长度
pub async fn llen(key: &str) -> RedisResult<usize> {
    if key.is_empty() {
        return Err(RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid key",
        )));
    }

    let mut conn = get_conn(key).await?;
    conn.llen(key)
}

// 获取列表指定索引的元素
pub async fn lindex<T: DeserializeOwned>(key: &str, index: isize) -> RedisResult<T> {
    if key.is_empty() {
        return Err(RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid key",
        )));
    }

    let mut conn = get_conn(key).await?;
    let val_json: String = conn.lindex(key, index)?;
    json_to_obj(&val_json).map_err(|e| RedisError::from(e))
}

// 设置列表指定索引的元素
pub async fn lset<T: Serialize>(key: &str, index: isize, value: &T) -> RedisResult<()> {
    if key.is_empty() {
        return Err(RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid key",
        )));
    }

    let val_json = obj_to_json(value)?;
    let mut conn = get_conn(key).await?;
    conn.lset(key, index, val_json)
}

// 获取列表指定范围的元素
pub async fn lrange<T: DeserializeOwned>(
    key: &str,
    start: isize,
    end: isize,
) -> RedisResult<Vec<T>> {
    if key.is_empty() {
        return Err(RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid key",
        )));
    }

    let mut conn = get_conn(key).await?;
    let values: Vec<String> = conn.lrange(key, start, end)?;
    values
        .into_iter()
        .map(|v| json_to_obj(&v).map_err(|e| RedisError::from(e)))
        .collect()
}

// 修剪列表，保留指定范围的元素
pub async fn ltrim(key: &str, start: isize, end: isize) -> RedisResult<()> {
    if key.is_empty() {
        return Err(RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "invalid key",
        )));
    }

    let mut conn = get_conn(key).await?;
    conn.ltrim(key, start, end)
}
