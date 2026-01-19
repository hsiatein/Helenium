use anyhow::Context;
use anyhow::Result;
use heleny_proto::HelenyFile;
use image::ColorType;
use image::GenericImageView;
use image::codecs::jpeg::JpegEncoder;
use std::path::PathBuf;
use std::time::SystemTime;
use tokio::fs::read_to_string;
use tokio::fs::{self};

#[derive(Clone)]
pub struct CacheEntry {
    pub content: HelenyFile,
    pub last_modified: SystemTime,
}

impl CacheEntry {
    pub async fn read_text(path: &PathBuf) -> Result<Self> {
        let content = read_to_string(path).await.context("读取文件失败")?;
        let last_modified: SystemTime = fs::metadata(path)
            .await
            .context("获取文件元数据失败")?
            .modified()
            .context("获取文件修改时间失败")?;
        Ok(Self {
            content: HelenyFile::Text(content),
            last_modified,
        })
    }

    pub async fn read_image(path: &PathBuf) -> Result<Self> {
        let content = tokio::fs::read(path).await?;
        let last_modified: SystemTime = fs::metadata(path)
            .await
            .context("获取文件元数据失败")?
            .modified()
            .context("获取文件修改时间失败")?;
        Ok(Self {
            content: HelenyFile::Image(content),
            last_modified,
        })
    }
}

pub fn make_thumbnail(
    image_bytes: &[u8],
    max_edge: u32,
) -> anyhow::Result<Vec<u8>> {
    // 解码
    let img = image::load_from_memory(image_bytes)?;

    let (w, h) = img.dimensions();
    let scale = max_edge as f32 / (w.max(h) as f32);
    let scale = scale.min(1.0); // 不放大

    let new_w = (w as f32 * scale).round() as u32;
    let new_h = (h as f32 * scale).round() as u32;

    // resize（质量优先）
    let thumb = img.resize(
        new_w,
        new_h,
        image::imageops::FilterType::Lanczos3,
    );

    // 转成 RGB8（JPEG 需要）
    let rgb = thumb.to_rgb8();

    let mut out = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut out, 80);
    encoder.encode(
        &rgb,
        rgb.width(),
        rgb.height(),
        ColorType::Rgb8.into(),
    )?;

    Ok(out)
}
