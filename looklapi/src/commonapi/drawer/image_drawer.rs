use crate::commonapi::drawer::image_helper::{load_image, load_image_async};
use crate::model::modelimpl::draw::{
    ImageContentModel, LineContentModel, RectangleContentModel, TextModel,
};
use image::{ImageBuffer, Rgba, RgbaImage};

/// 基于image库的绘图实现
pub struct ImageDrawer {
    image: Option<RgbaImage>,
}

impl ImageDrawer {
    /// 创建新实例
    pub fn new() -> Self {
        ImageDrawer { image: None }
    }

    /// 获取当前image
    fn get_image(&self) -> anyhow::Result<&RgbaImage> {
        self.image
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Image not initialized"))
    }

    /// 获取当前image的可变引用
    fn get_mut_image(&mut self) -> anyhow::Result<&mut RgbaImage> {
        self.image
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Image not initialized"))
    }

    /// 将AGBColor转换为Rgba
    pub fn to_rgba(agb_color: &[i32], transparent: i32) -> Rgba<u8> {
        let r = agb_color.get(0).copied().unwrap_or(0).clamp(0, 255) as u8;
        let g = agb_color.get(1).copied().unwrap_or(0).clamp(0, 255) as u8;
        let b = agb_color.get(2).copied().unwrap_or(0).clamp(0, 255) as u8;
        let a = if transparent == 0 {
            255
        } else {
            transparent.clamp(0, 255) as u8
        };
        Rgba([r, g, b, a])
    }

    /// 初始化绘图上下文
    pub fn init(&mut self, width: i32, height: i32) -> anyhow::Result<()> {
        let width = width.max(1) as u32;
        let height = height.max(1) as u32;
        self.image = Some(ImageBuffer::new(width, height));
        Ok(())
    }

    /// 保存图片到文件
    pub fn save_to_file(&self, filename: &str) -> anyhow::Result<()> {
        let image = self.get_image()?;
        image.save(filename)?;
        Ok(())
    }

    /// 生成图片字节流
    pub fn encode_to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let image = self.get_image()?;
        let mut buffer = Vec::new();
        image.write_to(
            &mut std::io::Cursor::new(&mut buffer),
            image::ImageFormat::Png,
        )?;
        Ok(buffer)
    }

    /// 绘制背景模板
    pub fn draw_template(&mut self, template: ImageContentModel) -> anyhow::Result<()> {
        let image = self.get_mut_image()?;

        // 填充背景色
        let color = Self::to_rgba(
            &template
                .rectangle_content_model
                .draw_content_model
                .agb_color,
            template.rectangle_content_model.transparent,
        );

        for x in 0..image.width() {
            for y in 0..image.height() {
                image.put_pixel(x, y, color);
            }
        }

        // 如果有图片路径，绘制图片作为背景
        if !template.path.is_empty() {
            if let Ok(img) = load_image(&template.path) {
                let dest_width = image.width();
                let dest_height = image.height();

                let resized = image::imageops::resize(
                    &img,
                    dest_width,
                    dest_height,
                    image::imageops::FilterType::Lanczos3,
                );

                // 将调整后的图片绘制到背景上
                for x in 0..dest_width {
                    for y in 0..dest_height {
                        if let Some(pixel) = resized.get_pixel_checked(x, y) {
                            image.put_pixel(x, y, *pixel);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// 绘制矩形背景区块
    pub fn draw_rectangle(&mut self, rect: RectangleContentModel) -> anyhow::Result<()> {
        let image = self.get_mut_image()?;

        let color = Self::to_rgba(&rect.draw_content_model.agb_color, rect.transparent);
        let x = rect.draw_content_model.x.max(0) as u32;
        let y = rect.draw_content_model.y.max(0) as u32;
        let width = rect.width.max(1) as u32;
        let height = rect.height.max(1) as u32;

        // 绘制矩形
        for px in x..(x + width) {
            for py in y..(y + height) {
                if px < image.width() && py < image.height() {
                    image.put_pixel(px, py, color);
                }
            }
        }

        Ok(())
    }

    /// 绘制线条
    pub fn draw_line(&mut self, line: LineContentModel) -> anyhow::Result<()> {
        let image = self.get_mut_image()?;

        let color = Self::to_rgba(&line.draw_content_model.agb_color, 255);
        let x = line.draw_content_model.x.max(0) as u32;
        let y = line.draw_content_model.y.max(0) as u32;
        let length = line.length.max(1) as u32;
        let line_width = line.width.max(1) as u32;

        // 绘制线条
        for wx in 0..line_width {
            for lx in 0..length {
                let px = x + lx;
                let py = y + wx;
                if px < image.width() && py < image.height() {
                    image.put_pixel(px, py, color);
                }
            }
        }

        Ok(())
    }

    /// 绘制图片
    pub fn draw_image(&mut self, img: ImageContentModel) -> anyhow::Result<()> {
        let image = self.get_mut_image()?;

        if let Ok(src_img) = load_image(&img.path) {
            let x = img.rectangle_content_model.draw_content_model.x.max(0) as u32;
            let y = img.rectangle_content_model.draw_content_model.y.max(0) as u32;
            let width = img.rectangle_content_model.width.max(1) as u32;
            let height = img.rectangle_content_model.height.max(1) as u32;

            let resized = image::imageops::resize(
                &src_img,
                width,
                height,
                image::imageops::FilterType::Lanczos3,
            );

            // 将图片绘制到指定位置
            for rx in 0..width {
                for ry in 0..height {
                    let px = x + rx;
                    let py = y + ry;
                    if px < image.width() && py < image.height() {
                        if let Some(pixel) = resized.get_pixel_checked(rx, ry) {
                            image.put_pixel(px, py, *pixel);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// 绘制二维码
    pub fn draw_qr_code(&mut self, qr_code: ImageContentModel) -> anyhow::Result<()> {
        // 二维码绘制与普通图片相同
        self.draw_image(qr_code)
    }

    /// 绘制头像
    pub fn draw_head_img(&mut self, head_img: ImageContentModel) -> anyhow::Result<()> {
        // 头像绘制与普通图片相同
        self.draw_image(head_img)
    }

    /// 绘制文字
    pub fn draw_text(&mut self, text: TextModel) -> anyhow::Result<()> {
        if !text.is_print {
            return Ok(());
        }

        let image = self.get_mut_image()?;

        // 尝试使用fontdue库和真实字体渲染文字
        // 如果没有字体文件，则使用简化版本
        match Self::render_text_with_fontdue(image, &text) {
            Ok(_) => Ok(()),
            Err(_) => {
                // 使用简化版本作为 fallback
                Self::render_text_simple(image, &text)
            }
        }
    }

    /// 使用fontdue库渲染文字
    fn render_text_with_fontdue(image: &mut RgbaImage, text: &TextModel) -> anyhow::Result<()> {
        // 尝试加载字体文件
        let font_path = "fonts/arial.ttf";
        
        if std::path::Path::new(font_path).exists() {
            let font_data = std::fs::read(font_path)?;
            let font = fontdue::Font::from_bytes(font_data.as_slice(), fontdue::FontSettings::default())
                .map_err(|e| anyhow::anyhow!("Failed to load font: {}", e))?;

            // 计算文字大小
            let font_size = text.size;

            // 处理文字对齐
            let x = text.draw_content_model.x as u32;
            let y = text.draw_content_model.y as u32;

            // 计算文字宽度
            let text_width = (text.text.len() as f32 * font_size * 0.6) as u32;

            // 处理文字对齐
            let mut draw_x = x;
            match text.alignment {
                1 => draw_x = x.saturating_sub(text_width / 2), // 居中对齐
                2 => draw_x = x.saturating_sub(text_width),     // 右对齐
                _ => {}                                         // 左对齐
            }

            // 绘制文字到图片
            let color = ImageDrawer::to_rgba(&text.draw_content_model.agb_color, 255);
            
            // 遍历文本中的每个字符并渲染
            let mut current_x = draw_x;
            for c in text.text.chars() {
                let (metrics, bitmap) = font.rasterize(c, font_size);

                let gx = current_x;
                let gy = y.saturating_sub(metrics.height as u32);

                // 将文字绘制到图片上
                for (by, row) in bitmap.chunks(metrics.width).enumerate() {
                    for (bx, &alpha) in row.iter().enumerate() {
                        if alpha > 0 {
                            let px = gx + bx as u32;
                            let py = gy + by as u32;

                            if px < image.width() && py < image.height() {
                                let mut pixel_color = color;
                                pixel_color.0[3] = alpha;
                                image.put_pixel(px, py, pixel_color);
                            }
                        }
                    }
                }

                // 移动到下一个字符的位置
                current_x += (metrics.advance_width as f32 * 0.6) as u32;
            }

            Ok(())
        } else {
            Err(anyhow::anyhow!("Font file not found"))
        }
    }

    /// 使用简化版本渲染文字
    fn render_text_simple(image: &mut RgbaImage, text: &TextModel) -> anyhow::Result<()> {
        let x = text.draw_content_model.x as u32;
        let y = text.draw_content_model.y as u32;
        let color = ImageDrawer::to_rgba(&text.draw_content_model.agb_color, 255);

        // 计算文字边界
        let font_size = text.size as u32;
        let text_width = (text.text.len() as f32 * text.size * 0.6) as u32;
        let _text_height = font_size;

        // 处理文字对齐
        let mut draw_x = x;
        match text.alignment {
            1 => draw_x = x.saturating_sub(text_width / 2), // 居中对齐
            2 => draw_x = x.saturating_sub(text_width),     // 右对齐
            _ => {}                                         // 左对齐
        }

        // 绘制文字的简化版本
        for (i, _c) in text.text.chars().enumerate() {
            let char_x = draw_x + (i as u32) * (font_size / 2);
            let char_y = y;

            // 绘制一个简单的矩形来表示字符
            for px in char_x..(char_x + font_size / 2) {
                for py in char_y..(char_y + font_size) {
                    if px < image.width() && py < image.height() {
                        image.put_pixel(px, py, color);
                    }
                }
            }
        }

        Ok(())
    }
}

/// 创建基于image库的绘图实例
pub fn new_image_drawer() -> ImageDrawer {
    ImageDrawer::new()
}

/// ImageDrawer的异步方法扩展
impl ImageDrawer {
    /// 绘制背景模板（异步）
    pub async fn draw_template_async(&mut self, template: ImageContentModel) -> anyhow::Result<()> {
        let image = self.get_mut_image()?;

        // 填充背景色
        let color = Self::to_rgba(
            &template
                .rectangle_content_model
                .draw_content_model
                .agb_color,
            template.rectangle_content_model.transparent,
        );

        for x in 0..image.width() {
            for y in 0..image.height() {
                image.put_pixel(x, y, color);
            }
        }

        // 如果有图片路径，绘制图片作为背景
        if !template.path.is_empty() {
            if let Ok(img) = load_image_async(&template.path).await {
                let dest_width = image.width();
                let dest_height = image.height();

                let resized = image::imageops::resize(
                    &img,
                    dest_width,
                    dest_height,
                    image::imageops::FilterType::Lanczos3,
                );

                // 将调整后的图片绘制到背景上
                for x in 0..dest_width {
                    for y in 0..dest_height {
                        if let Some(pixel) = resized.get_pixel_checked(x, y) {
                            image.put_pixel(x, y, *pixel);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// 绘制图片（异步）
    pub async fn draw_image_async(&mut self, img: ImageContentModel) -> anyhow::Result<()> {
        let image = self.get_mut_image()?;

        if let Ok(src_img) = load_image_async(&img.path).await {
            let x = img.rectangle_content_model.draw_content_model.x.max(0) as u32;
            let y = img.rectangle_content_model.draw_content_model.y.max(0) as u32;
            let width = img.rectangle_content_model.width.max(1) as u32;
            let height = img.rectangle_content_model.height.max(1) as u32;

            let resized = image::imageops::resize(
                &src_img,
                width,
                height,
                image::imageops::FilterType::Lanczos3,
            );

            // 将图片绘制到指定位置
            for rx in 0..width {
                for ry in 0..height {
                    let px = x + rx;
                    let py = y + ry;
                    if px < image.width() && py < image.height() {
                        if let Some(pixel) = resized.get_pixel_checked(rx, ry) {
                            image.put_pixel(px, py, *pixel);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// 绘制二维码（异步）
    pub async fn draw_qr_code_async(&mut self, qr_code: ImageContentModel) -> anyhow::Result<()> {
        // 二维码绘制与普通图片相同
        self.draw_image_async(qr_code).await
    }

    /// 绘制头像（异步）
    pub async fn draw_head_img_async(&mut self, head_img: ImageContentModel) -> anyhow::Result<()> {
        // 头像绘制与普通图片相同
        self.draw_image_async(head_img).await
    }

    /// 绘制文字（异步）
    pub async fn draw_text_async(&mut self, text: TextModel) -> anyhow::Result<()> {
        // 文字绘制与同步版本相同，因为不需要加载外部资源
        self.draw_text(text)
    }
}
