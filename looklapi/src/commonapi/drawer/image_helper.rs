use image::DynamicImage;

/// 加载图片（同步版本）
pub fn load_image(path: &str) -> anyhow::Result<DynamicImage> {
    if path.starts_with("http://") || path.starts_with("https://") {
        // 从URL加载
        load_image_from_url(path)
    } else if path.starts_with("data:image/") {
        // 从base64加载
        load_image_from_base64(path)
    } else {
        // 从本地文件加载
        load_image_from_file(path)
    }
}

/// 加载图片（异步版本）
pub async fn load_image_async(path: &str) -> anyhow::Result<DynamicImage> {
    if path.starts_with("http://") || path.starts_with("https://") {
        // 从URL加载
        load_image_from_url_async(path).await
    } else if path.starts_with("data:image/") {
        // 从base64加载
        load_image_from_base64(path)
    } else {
        // 从本地文件加载
        load_image_from_file(path)
    }
}

/// 从URL加载图片（同步版本）
fn load_image_from_url(url: &str) -> anyhow::Result<DynamicImage> {
    let response = reqwest::blocking::get(url)?;
    let bytes = response.bytes()?;
    let img = image::load_from_memory(&bytes)?;
    Ok(img)
}

/// 从URL加载图片（异步版本）
async fn load_image_from_url_async(url: &str) -> anyhow::Result<DynamicImage> {
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;
    let img = image::load_from_memory(&bytes)?;
    Ok(img)
}

/// 从base64加载图片
fn load_image_from_base64(base64_str: &str) -> anyhow::Result<DynamicImage> {
    // 移除data:image/xxx;base64,前缀
    let data = base64_str.split(",").nth(1).ok_or_else(|| anyhow::anyhow!("Invalid base64 format"))?;
    let bytes = base64::decode(data)?;
    let img = image::load_from_memory(&bytes)?;
    Ok(img)
}

/// 从本地文件加载图片
fn load_image_from_file(path: &str) -> anyhow::Result<DynamicImage> {
    let img = image::open(path)?;
    Ok(img)
}

/// 调整图片大小
pub fn resize_image(img: &DynamicImage, width: u32, height: u32) -> DynamicImage {
    let resized = image::imageops::resize(img, width, height, image::imageops::FilterType::Lanczos3);
    DynamicImage::ImageRgba8(resized)
}

/// 使用cover填充方式处理图片
pub fn cover_image(img: &DynamicImage, width: u32, height: u32) -> DynamicImage {
    let img_width = img.width();
    let img_height = img.height();
    
    let img_ratio = img_width as f32 / img_height as f32;
    let target_ratio = width as f32 / height as f32;
    
    if img_ratio > target_ratio {
        // 图片更宽，按高度缩放，然后裁剪宽度
        let new_height = height;
        let new_width = (height as f32 * img_ratio) as u32;
        let resized = image::imageops::resize(img, new_width, new_height, image::imageops::FilterType::Lanczos3);
        let x = (new_width - width) / 2;
        let cropped = image::imageops::crop(&mut resized.clone(), x, 0, width, height).to_image();
        DynamicImage::ImageRgba8(cropped)
    } else {
        // 图片更高，按宽度缩放，然后裁剪高度
        let new_width = width;
        let new_height = (width as f32 / img_ratio) as u32;
        let resized = image::imageops::resize(img, new_width, new_height, image::imageops::FilterType::Lanczos3);
        let y = (new_height - height) / 2;
        let cropped = image::imageops::crop(&mut resized.clone(), 0, y, width, height).to_image();
        DynamicImage::ImageRgba8(cropped)
    }
}
