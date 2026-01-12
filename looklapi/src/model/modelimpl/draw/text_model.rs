use super::draw_content_model::DrawContentModel;

/// 动态文字模型
#[derive(Debug, Default)]
pub struct DynamicTextModel {
    // 这里可以根据实际需要添加字段
}

/// 静态文字模型
#[derive(Debug, Default)]
pub struct TextModel {
    /// 基础绘图模型
    pub draw_content_model: DrawContentModel,
    /// 是否打印
    pub is_print: bool,
    /// 文字内容
    pub text: String,
    /// 字体
    pub font: String,
    /// 文字大小
    pub size: f32,
    /// 是否加粗
    pub is_bold: bool,
    /// 是否倾斜
    pub is_italic: bool,
    /// 水平间距
    pub h_gap: f32,
    /// 行距
    pub v_gap: f32,
    /// 对齐方式
    pub alignment: i32,
    /// 边界横向坐标
    pub boundary: i32,
    /// 行数限制
    pub row_limit: i32,
    /// 关联动态字符
    pub related_dynamic_text: Vec<Box<DynamicTextModel>>,
}

impl TextModel {
    /// 创建静态文字模型实例
    pub fn new() -> Self {
        TextModel {
            draw_content_model: DrawContentModel::default(),
            related_dynamic_text: Vec::new(),
            ..Default::default()
        }
    }
}
