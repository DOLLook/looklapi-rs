use super::draw_content_model::DrawContentModel;

/// 线段模型
#[derive(Debug, Default)]
pub struct LineContentModel {
    /// 基础绘图模型
    pub draw_content_model: DrawContentModel,
    /// 线宽
    pub width: i32,
    /// 长度
    pub length: i32,
}
