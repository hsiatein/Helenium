use anyhow::Context;
use anyhow::Result;
use base64::prelude::*;
use heleny_proto::ChatRole;
use heleny_proto::ConsentRequestionFE;
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
use slint::Weak;
use tokio::sync::mpsc;
use tracing::debug;
use tungstenite::Message;
mod handle_ws;
pub use handle_ws::*;
mod set_callback;
use image::DynamicImage;
pub use set_callback::*;
mod terminal;
use terminal::*;

slint::include_modules!();

pub struct FrontendHandler {
    writer: mpsc::Sender<Message>,
    ui_weak: Weak<AppWindow>,
}

impl FrontendHandler {
    pub fn new(writer: mpsc::Sender<Message>, ui_weak: Weak<AppWindow>) -> Self {
        Self { writer, ui_weak }
    }
    pub async fn handle_frontend_message(&self, msg: FrontendMessage) -> Result<()> {
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
                        if let MemoryContent::Image(path) = &message.content {
                            let _ = self
                                .writer
                                .send(
                                    FrontendCommand::GetImage {
                                        id: message.id,
                                        path: path.clone(),
                                    }
                                    .into(),
                                )
                                .await;
                        }
                    }
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
                                        MemoryContent::Text(text) => Some(MessageItem {
                                            id: id as i32,
                                            is_me: role != ChatRole::Assistant,
                                            kind: "text".into(),
                                            text: text.into(),
                                            image: Image::default(),
                                            time: time
                                                .format("%Y-%m-%d %H:%M:%S")
                                                .to_string()
                                                .into(),
                                        }),
                                        MemoryContent::Image(_) => Some(MessageItem {
                                            id: id as i32,
                                            is_me: role != ChatRole::Assistant,
                                            kind: "image".into(),
                                            text: "".into(),
                                            image: ui.get_default_image(),
                                            time: time
                                                .format("%Y-%m-%d %H:%M:%S")
                                                .to_string()
                                                .into(),
                                        }),
                                    }
                                })
                                .collect();
                            if !new {
                                ui.invoke_prepare_history_scroll();
                            }

                            let mut history: Vec<MessageItem> =
                                ui.get_chat_model().iter().collect();
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
                ResourcePayload::Image { id, base64 } => {
                    let image_u8 = BASE64_STANDARD.decode(base64)?;
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
                                ),
                            );
                            let mut model: Vec<MessageItem> = ui.get_chat_model().iter().collect();
                            if let Some(item) = model.iter_mut().find(|msg| msg.id as i64 == id) {
                                item.image = slint_img;
                            }
                            let model = ModelRc::new(slint::VecModel::from(model));
                            ui.set_chat_model(model);
                        })
                        .context("更新图片失败")?;
                }
            },
            FrontendMessage::UserDecision(user_decison) => match user_decison {
                UserDecision::ConsentRequestions(consent_requestions) => {
                    debug!("{:?}", consent_requestions);
                    self.ui_weak
                        .upgrade_in_event_loop(move |ui| {
                            let mut reqs: Vec<ConsentRequestionSlint> =
                                ui.get_consent_requestions().iter().collect();
                            let new_reqs: Vec<ConsentRequestionSlint> = consent_requestions
                                .into_iter()
                                .map(|req_fe| {
                                    let ConsentRequestionFE {
                                        request_id,
                                        task_id,
                                        task_description,
                                        reason,
                                        descripion,
                                    } = req_fe;
                                    ConsentRequestionSlint {
                                        descripion: descripion.into(),
                                        reason: reason.into(),
                                        request_id: request_id.to_string().into(),
                                        task_description: task_description.into(),
                                        task_id: task_id.to_string().into(),
                                    }
                                })
                                .collect();
                            reqs.extend(new_reqs);
                            ui.set_consent_requestions(ModelRc::new(slint::VecModel::from(reqs)));
                        })
                        .context("更新审批失败")?;
                }
            },
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
