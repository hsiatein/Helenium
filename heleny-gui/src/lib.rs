use anyhow::Context;
use anyhow::Result;
use chrono::DateTime;
use chrono::Local;
use chrono::TimeZone;
use heleny_proto::ChatRole;
use heleny_proto::DisplayMessage;
use heleny_proto::FrontendCommand;
use heleny_proto::FrontendMessage;
use heleny_proto::MemoryContent;
use heleny_proto::ResourcePayload;
use heleny_proto::UserDecision;
use slint::Image;
use slint::Model;
use slint::ModelRc;
use slint::Rgba8Pixel;
use slint::SharedPixelBuffer;
use slint::VecModel;
use slint::Weak;
use tokio::sync::mpsc;
use tungstenite::Message;
use std::collections::VecDeque;
use base64::prelude::*;
use std::fmt::Write;
use tracing::debug;
mod handle_ws;
pub use handle_ws::*;
mod set_callback;
pub use set_callback::*;
use image::DynamicImage;

slint::include_modules!();

pub struct FrontendHandler {
    writer: mpsc::Sender<Message>, 
    ui_weak: Weak<AppWindow>,
}

impl FrontendHandler {
    pub fn new(writer: mpsc::Sender<Message>,ui_weak: Weak<AppWindow> )->Self{
        Self {writer,ui_weak}
    }
    pub async fn handle_frontend_message(&self, msg: FrontendMessage, ) -> Result<()> {
        match msg {
            FrontendMessage::UpdateResource(resource) => match resource.payload {
                ResourcePayload::TotalBusTraffic(data) => {
                    let (svg, y_max, x_start, x_end) = generate_svg_path(&data, 300., 200.);
                    self.ui_weak
                        .upgrade_in_event_loop(move |ui| {
                            ui.set_bus_stats_chart(svg.into());
                            ui.set_bus_y_max(y_max.into());
                            ui.set_bus_x_start(x_start.into());
                            ui.set_bus_x_end(x_end.into());
                        })
                        .context("绘图 bus_stats_chart 失败")?;
                }
                ResourcePayload::DisplayMessages { new, messages } => {
                    debug!("{:?}", messages);
                    for message in &messages {
                        if let MemoryContent::Image(path)= &message.content {
                            let _=self.writer.send(FrontendCommand::GetImage { id: message.id, path:path.clone() }.into()).await;
                        }
                    };
                    self.ui_weak
                        .upgrade_in_event_loop(move |ui| {
                            let mut messages: Vec<MessageItem> = messages
                            .into_iter()
                            .filter_map(|msg| {
                                let DisplayMessage {
                                    id,
                                    role,
                                    time,
                                    content,
                                } = msg;
                                match content {
                                    MemoryContent::Text(text) => {
                                        Some(MessageItem {
                                            id: id as i32,
                                            is_me: role != ChatRole::Assistant,
                                            kind: "text".into(),
                                            text: text.into(),
                                            image: Image::default(),
                                            time: time.format("%Y-%m-%d %H:%M:%S").to_string().into(),
                                        })
                                    }
                                    MemoryContent::Image(_) => {
                                        Some(MessageItem {
                                            id: id as i32,
                                            is_me: role != ChatRole::Assistant,
                                            kind: "image".into(),
                                            text: "".into(),
                                            image: ui.get_default_image(),
                                            time: time.format("%Y-%m-%d %H:%M:%S").to_string().into(),
                                        })
                                    }
                                }
                                
                            })
                            .collect();
                            if !new {
                                ui.invoke_prepare_history_scroll();
                            }

                            let mut history: Vec<MessageItem> = ui.get_chat_model().iter().collect();
                            let history = if new {
                                history.extend(messages);
                                history
                            } else {
                                messages.extend(history);
                                messages
                            };
                            let model = ModelRc::new(slint::VecModel::from(history));
                            ui.set_chat_model(model);

                            if new {
                                ui.invoke_scroll_to_bottom();
                            } else {
                                ui.invoke_finish_history_scroll();
                            }
                        })
                        .context("绘图 bus_stats_chart 失败")?;
                }
                ResourcePayload::Health(health) => {
                    debug!("{:?}", health);
                    let mut services: Vec<ServiceHealthItem> = health
                        .services
                        .into_iter()
                        .map(|(name, (status, _))| {
                            let status_str = match status {
                                heleny_proto::HealthStatus::Starting => "Starting",
                                heleny_proto::HealthStatus::Healthy => "Healthy",
                                heleny_proto::HealthStatus::Unhealthy => "Unhealthy",
                                heleny_proto::HealthStatus::Stopping => "Stopping",
                                heleny_proto::HealthStatus::Stopped => "Stopped",
                            };
                            ServiceHealthItem {
                                name: name.into(),
                                status: status_str.into(),
                            }
                        })
                        .collect();

                    services.sort_by(|a, b| a.name.cmp(&b.name));

                    self.ui_weak
                        .upgrade_in_event_loop(move |ui| {
                            let model = ModelRc::new(slint::VecModel::from(services));
                            ui.set_services_health(model);
                        })
                        .context("更新服务健康度失败")?;
                }
                ResourcePayload::Image { id, base64 }=>{
                    let image_u8=BASE64_STANDARD.decode(base64)?;
                    let img: DynamicImage = image::load_from_memory(&image_u8)?;
                    let rgba = img.to_rgba8();
                    let (w, h) = rgba.dimensions();
                    self.ui_weak
                        .upgrade_in_event_loop(move |ui| {
                            let slint_img = slint::Image::from_rgba8(
                                SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                                    rgba.as_raw(),
                                    w,
                                    h,
                                )
                            );
                            let mut model:Vec<MessageItem>=ui.get_chat_model().iter().collect();
                            if let Some(item)=model.iter_mut().find(|msg| msg.id as i64==id) {
                                item.image=slint_img;
                            }
                            let model=ModelRc::new(slint::VecModel::from(model));
                            ui.set_chat_model(model);
                        })
                        .context("更新图片失败")?;
                }
            },
            FrontendMessage::UserDecision(user_decison)=>{
                match user_decison {
                    UserDecision::ConsentRequestions(consent_requestions)=>{
                        
                    }
                }
            }
        }
        Ok(())
    }

}


fn generate_svg_path(
    data: &VecDeque<(DateTime<Local>, usize)>,
    width: f32,
    height: f32,
) -> (String, String, String, String) {
    if data.is_empty() {
        return (
            String::new(),
            "0".into(),
            "00:00:00".into(),
            "00:00:00".into(),
        );
    }

    // 1. Sort by timestamp
    let mut raw_points: Vec<_> = data.iter().collect();
    raw_points.sort_by_key(|(time, _)| time);

    // 2. Deduplicate: Merge points with same timestamp (keep max value)
    // points will store (timestamp_millis, traffic_value)
    let mut points: Vec<(i64, usize)> = Vec::with_capacity(raw_points.len());

    if let Some((first_time, first_val)) = raw_points.first() {
        let mut current_time = first_time.timestamp_millis();
        let mut max_val_for_time = *first_val;

        for (time, val) in raw_points.iter().skip(1) {
            let t_millis = time.timestamp_millis();
            let v = *val;

            if t_millis == current_time {
                // Same timestamp: keep the larger value
                if v > max_val_for_time {
                    max_val_for_time = v;
                }
            } else {
                // New timestamp: push previous and reset
                points.push((current_time, max_val_for_time));
                current_time = t_millis;
                max_val_for_time = v;
            }
        }
        // Push the last accumulated point
        points.push((current_time, max_val_for_time));
    }

    if points.is_empty() {
        return (
            String::new(),
            "0".into(),
            "00:00:00".into(),
            "00:00:00".into(),
        );
    }

    // debug!("BusTraffic Graph: {} points after dedup (raw {})", points.len(), data.len());

    // 3. Calculate Range
    let min_time = points.first().unwrap().0;
    let max_time = points.last().unwrap().0;
    let time_range = (max_time - min_time).max(1) as f32;

    let min_val = 0.0;
    let max_traffic = points.iter().map(|(_, v)| *v).max().unwrap_or(0);
    // Fix: Remove the 1.2 multiplier so the graph fills the height.
    // Ensure max_val is at least 1.0 to avoid flat lines or div by zero if traffic is 0.
    let max_val = (max_traffic as f32).max(10.0);
    let val_range = (max_val - min_val).max(1.0);

    let mut path = String::with_capacity(points.len() * 30);

    for (i, (time_millis, val)) in points.iter().enumerate() {
        // 归一化并映射到画布坐标
        // Fix: Subtract integers first to preserve precision. converting a huge timestamp to f32 loses precision.
        let time_diff = (*time_millis - min_time) as f32;
        let v_float = *val as f32;

        let x = (time_diff / time_range) * width;
        // 翻转 Y 轴：数值大 -> 坐标小(上方)
        let y = height * (1.0 - (v_float - min_val) / val_range);

        if i == 0 {
            write!(path, "M {:.1} {:.1}", x, y).unwrap();
        } else {
            write!(path, " L {:.1} {:.1}", x, y).unwrap();
        }
    }

    let min_time_dt = Local.timestamp_millis_opt(min_time).unwrap();
    let max_time_dt = Local.timestamp_millis_opt(max_time).unwrap();
    let x_start = min_time_dt.format("%H:%M:%S").to_string();
    let x_end = max_time_dt.format("%H:%M:%S").to_string();
    // Use max_val (the scaling ceiling) for the label, not just the max data point.
    let y_max = format!("{}", max_val as usize);

    (path, y_max, x_start, x_end)
}
