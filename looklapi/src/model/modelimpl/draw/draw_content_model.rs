/// 绘制图形基类
#[derive(Debug, Default)]
pub struct DrawContentModel {
    /// RGB颜色
    pub agb_color: Vec<i32>,
    /// X位置
    pub x: i32,
    /// Y位置
    pub y: i32,
}
