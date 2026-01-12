/// 文本行信息
pub struct TextLine {
    pub text: String,
    pub x: f32,
    pub y: f32,
}

/// 处理文本，分割为多行
pub fn process_text_for_drawing<F>(
    text: &str,
    max_width: f64,
    row_limit: i32,
    measure_width: F,
    _h_gap: f64,
) -> Vec<String>
where
    F: Fn(&str) -> f64,
{
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0.0;
    
    for word in text.split_whitespace() {
        let word_width = measure_width(word);
        let space_width = measure_width(" ");
        
        if current_line.is_empty() {
            // 第一个单词
            current_line.push_str(word);
            current_width = word_width;
        } else {
            // 计算添加空格和新单词后的宽度
            let new_width = current_width + space_width + word_width;
            
            if new_width <= max_width {
                // 可以添加到当前行
                current_line.push(' ');
                current_line.push_str(word);
                current_width = new_width;
            } else {
                // 需要换行
                lines.push(current_line.clone());
                if lines.len() as i32 >= row_limit && row_limit > 0 {
                    break;
                }
                current_line = word.to_string();
                current_width = word_width;
            }
        }
    }
    
    if !current_line.is_empty() && ((lines.len() as i32) < row_limit || row_limit <= 0) {
        lines.push(current_line);
    }
    
    lines
}

/// 计算文本行的位置
pub fn calculate_text_lines_position<F1, F2>(
    lines: Vec<String>,
    x: i32,
    y: i32,
    max_width: i32,
    _h_gap: f64,
    alignment: i32,
    v_gap: f64,
    measure_width: F1,
    measure_height: F2,
) -> Vec<TextLine>
where
    F1: Fn(&str) -> f64,
    F2: Fn() -> f64,
{
    let mut text_lines = Vec::new();
    let mut current_y = y as f32;
    let line_height = measure_height() as f32;
    
    for line in lines {
        let line_width = measure_width(&line) as f32;
        let mut line_x = x as f32;
        
        // 根据对齐方式计算x坐标
        match alignment {
            1 => { // 居中对齐
                line_x += (max_width as f32 - line_width) / 2.0;
            }
            2 => { // 右对齐
                line_x += max_width as f32 - line_width;
            }
            _ => { // 左对齐
                // 保持默认x坐标
            }
        }
        
        text_lines.push(TextLine {
            text: line,
            x: line_x,
            y: current_y,
        });
        
        current_y += line_height + v_gap as f32;
    }
    
    text_lines
}
