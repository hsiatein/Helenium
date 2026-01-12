use anyhow::Result;
use heleny_proto::ResourcePayload;
use std::collections::HashMap;
use std::str::FromStr;

use crate::FrontendHandler;
use crate::MessageItem;
use crate::ScheduleItem;
use crate::ServiceHealthItem;
use crate::TaskItem;
use crate::terminal::generate_svg_path;
use anyhow::Context;
use base64::prelude::*;
use heleny_proto::ChatRole;
use heleny_proto::DisplayMessage;
use heleny_proto::FrontendCommand;
use heleny_proto::MemoryContent;
use heleny_proto::TaskAbstract;
use image::DynamicImage;
use slint::Image;
use slint::Model;
use slint::ModelRc;
use slint::Rgba8Pixel;
use slint::SharedPixelBuffer;
use slint::SharedString;
use tracing::debug;
use uuid::Uuid;

impl FrontendHandler {
    pub async fn handle_resource(&self, resource: ResourcePayload) -> Result<()> {
        match resource {
            ResourcePayload::TotalBusTraffic(data) => {
                let (svg, y_max, y_mid, x_start, x_end) = generate_svg_path(&data, 600., 240.);
                self.ui_weak
                    .upgrade_in_event_loop(move |ui| {
                        ui.set_bus_stats_chart(svg.into());
                        ui.set_bus_y_max(y_max.into());
                        ui.set_bus_y_mid(y_mid.into());
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
                                        time: time.format("%Y-%m-%d %H:%M:%S").to_string().into(),
                                    }),
                                    MemoryContent::Image(_) => Some(MessageItem {
                                        id: id as i32,
                                        is_me: role != ChatRole::Assistant,
                                        kind: "image".into(),
                                        text: "".into(),
                                        image: ui.get_default_image(),
                                        time: time.format("%Y-%m-%d %H:%M:%S").to_string().into(),
                                    }),
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
            ResourcePayload::Image { id, base64 } => {
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
            }
            ResourcePayload::TaskAbstract { task_abstracts } => {
                debug!("任务摘要: {:?}", task_abstracts);
                let _ = self.ui_weak.upgrade_in_event_loop(|ui| {
                    let mut tasks: HashMap<Uuid, TaskItem> = ui
                        .get_tasks()
                        .iter()
                        .filter_map(|task| {
                            let Ok(id) = Uuid::from_str(task.id.as_str()) else {
                                return None;
                            };
                            Some((id, task))
                        })
                        .collect();
                    let new_tasks: Vec<TaskItem> = task_abstracts
                        .into_iter()
                        .map(
                            |TaskAbstract {
                                 id,
                                 task_description,
                                 status,
                             }| {
                                let old = tasks.remove(&id);
                                match old {
                                    Some(mut task) => {
                                        task.status = status.to_string().into();
                                        task
                                    }
                                    None => TaskItem {
                                        id: id.to_string().into(),
                                        task_description: task_description.into(),
                                        status: status.to_string().into(),
                                        expanded: false,
                                        logs: ModelRc::new(slint::VecModel::from(Vec::<
                                            SharedString,
                                        >::new(
                                        ))),
                                    },
                                }
                            },
                        )
                        .collect();
                    ui.set_tasks(ModelRc::new(slint::VecModel::from(new_tasks)));
                });
            }
            ResourcePayload::TaskLogs { id, logs } => {
                let _ = self.ui_weak.upgrade_in_event_loop(move |ui| {
                    let mut tasks: Vec<TaskItem> = ui.get_tasks().iter().collect();
                    let Some(task) = tasks.iter_mut().find(|task| {
                        let Ok(task_id) = Uuid::from_str(&task.id) else {
                            return false;
                        };
                        task_id == id
                    }) else {
                        return;
                    };
                    task.logs = ModelRc::new(slint::VecModel::from(
                        logs.into_iter()
                            .map(|log| log.into())
                            .collect::<Vec<SharedString>>(),
                    ));
                    ui.set_tasks(ModelRc::new(slint::VecModel::from(tasks)));
                });
            }
            ResourcePayload::Schedule { schedule } => {
                debug!("ResourcePayload::Schedule: {:?}", schedule);
                let _ = self.ui_weak.upgrade_in_event_loop(move |ui| {
                    let schedules=schedule.into_iter().map(|(id,task)|{
                        ScheduleItem{
                            id,description:task.description,next_trigger:task.
                        }
                    });
                    let schedules:Vec<ScheduleItem>=ui.get_schedules().iter().collect();
                });
            }
        }
        Ok(())
    }
}
