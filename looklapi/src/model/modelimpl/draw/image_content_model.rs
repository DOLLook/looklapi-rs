use super::rectangle_content_model::RectangleContentModel;

/// 贴图模型
#[derive(Debug, Default)]
pub struct ImageContentModel {
    /// 矩形内容模型
    pub rectangle_content_model: RectangleContentModel,
    /// 序列
    pub serial: i32,
    /// 图片 base64/url
    pub path: String,
}
