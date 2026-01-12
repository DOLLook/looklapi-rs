use super::draw_content_model::DrawContentModel;

/// 四边形内容模型
#[derive(Debug, Default)]
pub struct RectangleContentModel {
    /// 基础绘图模型
    pub draw_content_model: DrawContentModel,
    /// 宽度
    pub width: i32,
    /// 高度
    pub height: i32,
    /// 倒角半径
    pub radius: i32,
    /// 透明度(0~255)
    pub transparent: i32,
}
