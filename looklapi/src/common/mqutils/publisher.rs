use crate::common::mqutils::models::MqMessage;
use crate::common::mqutils::rabbitmq_pool::RabbitmqConnPool;
use chrono;
use lapin;
use serde::Serialize;
use serde_json;
use tracing;
use uuid::Uuid;

// 发布工作队列消息
pub async fn pub_work_queue_msg<T: Serialize>(route_key: &str, msg: T) -> bool {
    if route_key.is_empty() {
        return false;
    }

    let meta_msg = convert_message(msg);
    if meta_msg.is_none() {
        return false;
    }

    let json_msg = serde_json::to_string(&meta_msg.unwrap()).unwrap();
    if let Err(err) = pub_work_queue_msg_internal(route_key, &json_msg).await {
        tracing::error!("发布工作队列消息失败: {:?}", err);
        return false;
    }

    true
}

// 发布广播消息
pub async fn pub_broadcast_msg<T: Serialize>(exchange: &str, msg: T) -> bool {
    if exchange.is_empty() {
        return false;
    }

    let meta_msg = convert_message(msg);
    if meta_msg.is_none() {
        return false;
    }

    let json_msg = serde_json::to_string(&meta_msg.unwrap()).unwrap();
    if let Err(err) = pub_broadcast_msg_internal(exchange, &json_msg).await {
        tracing::error!("发布广播消息失败: {:?}", err);
        return false;
    }

    true
}

// 发布topic消息
pub async fn pub_topic_msg<T: Serialize>(exchange: &str, route_key: &str, msg: T) -> bool {
    if exchange.is_empty() || route_key.is_empty() {
        return false;
    }

    let meta_msg = convert_message(msg);
    if meta_msg.is_none() {
        return false;
    }

    let json_msg = serde_json::to_string(&meta_msg.unwrap()).unwrap();
    if let Err(err) = pub_topic_msg_internal(exchange, route_key, &json_msg).await {
        tracing::error!("发布topic消息失败: {:?}", err);
        return false;
    }

    true
}

// 发布工作队列消息内部实现
async fn pub_work_queue_msg_internal(
    route_key: &str,
    json_msg: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let pool = RabbitmqConnPool::get_instance();
    let pub_chan = pool.get_pub_channel().await?;

    // 声明队列
    pub_chan
        .channel
        .queue_declare(
            route_key,
            lapin::options::QueueDeclareOptions {
                durable: true,
                auto_delete: false,
                exclusive: false,
                ..Default::default()
            },
            lapin::types::FieldTable::default(),
        )
        .await?;

    // 发布消息
    pub_chan
        .channel
        .basic_publish(
            "",        // exchange
            route_key, // routing_key
            lapin::options::BasicPublishOptions::default(),
            json_msg.as_bytes(),
            lapin::BasicProperties::default()
                .with_content_type("application/octet-stream".into())
                .with_delivery_mode(2), // persistent
        )
        .await?;

    pool.release_channel(pub_chan);
    Ok(())
}

// 发布广播消息内部实现
async fn pub_broadcast_msg_internal(
    exchange: &str,
    json_msg: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let pool = RabbitmqConnPool::get_instance();
    let pub_chan = pool.get_pub_channel().await?;

    // 声明交换器
    pub_chan
        .channel
        .exchange_declare(
            exchange,
            lapin::ExchangeKind::Fanout,
            lapin::options::ExchangeDeclareOptions {
                durable: false,
                auto_delete: true,
                internal: false,
                ..Default::default()
            },
            lapin::types::FieldTable::default(),
        )
        .await?;

    // 发布消息
    pub_chan
        .channel
        .basic_publish(
            exchange, // exchange
            "",       // routing_key
            lapin::options::BasicPublishOptions::default(),
            json_msg.as_bytes(),
            lapin::BasicProperties::default().with_content_type("application/octet-stream".into()),
        )
        .await?;

    pool.release_channel(pub_chan);
    Ok(())
}

// 发布topic消息内部实现
async fn pub_topic_msg_internal(
    exchange: &str,
    route_key: &str,
    json_msg: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let pool = RabbitmqConnPool::get_instance();
    let pub_chan = pool.get_pub_channel().await?;

    // 声明交换器
    pub_chan
        .channel
        .exchange_declare(
            exchange,
            lapin::ExchangeKind::Topic,
            lapin::options::ExchangeDeclareOptions {
                durable: true,
                auto_delete: false,
                internal: false,
                ..Default::default()
            },
            lapin::types::FieldTable::default(),
        )
        .await?;

    // 发布消息
    pub_chan
        .channel
        .basic_publish(
            exchange,  // exchange
            route_key, // routing_key
            lapin::options::BasicPublishOptions::default(),
            json_msg.as_bytes(),
            lapin::BasicProperties::default()
                .with_content_type("application/octet-stream".into())
                .with_delivery_mode(2), // persistent
        )
        .await?;

    pool.release_channel(pub_chan);
    Ok(())
}

// 转换消息为MQ消息格式
fn convert_message<T: Serialize>(msg: T) -> Option<MqMessage> {
    let json_content = match serde_json::to_string(&msg) {
        Ok(content) => content,
        Err(err) => {
            tracing::error!("消息序列化失败: {:?}", err);
            return None;
        }
    };

    Some(MqMessage {
        guid: uuid::Uuid::new_v4().to_string().replace('-', ""),
        timespan: chrono::Utc::now(),
        current_retry: 0,
        json_content,
    })
}
