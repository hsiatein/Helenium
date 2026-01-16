use crate::FrontendHandler;
use crate::MessageItem;
use anyhow::Context;
use anyhow::Result;
use base64::prelude::*;
use image::DynamicImage;
use slint::Model;
use slint::ModelRc;
use slint::Rgba8Pixel;
use slint::SharedPixelBuffer;

impl FrontendHandler {
    pub async fn handle_image(&self, id: i64, base64: String) -> Result<()> {
        let image_u8 = BASE64_STANDARD.decode(base64)?;
        let img: DynamicImage = image::load_from_memory(&image_u8)?;
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        self.ui_weak
            .upgrade_in_event_loop(move |ui| {
                let slint_img = slint::Image::from_rgba8(
                    SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(rgba.as_raw(), w, h),
                );
                let mut model: Vec<MessageItem> = ui.get_chat_model().iter().collect();
                if let Some(item) = model.iter_mut().find(|msg| msg.id as i64 == id) {
                    item.image = slint_img;
                }
                let model = ModelRc::new(slint::VecModel::from(model));
                ui.set_chat_model(model);
            })
            .context("更新图片失败")?;
        Ok(())
    }
}
