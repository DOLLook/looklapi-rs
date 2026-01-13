use crate::common::mqutils::consts::ConsumerType;
use crate::common::mqutils::models::MqMessage;
use tracing;

// 重试消息
pub fn retry(meta_msg: &MqMessage, consumer: &crate::common::mqutils::consumer::Consumer) -> bool {
    if meta_msg.current_retry >= consumer.max_retry as i32 {
        tracing::error!("消息重试次数超过最大值: {:?}", meta_msg);
        return true;
    }

    // 这里可以实现消息重试的逻辑，例如将消息发送到重试队列
    // 由于我们使用的是RabbitMQ的内置重试机制（通过nack和requeue），这里可以简化处理
    false
}

// 重试成功
pub fn retry_success(meta_msg: &MqMessage, consumer_type: ConsumerType) {
    // 这里可以实现重试成功后的逻辑，例如更新重试状态等
    tracing::info!("消息处理成功: {} - {:?}", meta_msg.guid, consumer_type);
}
