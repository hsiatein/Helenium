use crate::FrontendHandler;
use crate::MessageItem;
use anyhow::Context;
use anyhow::Result;
use heleny_proto::ChatRole;
use heleny_proto::MemoryEntry;
use heleny_proto::FrontendCommand;
use heleny_proto::MemoryContent;
use slint::Image;
use slint::Model;
use slint::ModelRc;

impl FrontendHandler {
    pub async fn handle_display_messages(
        &self,
        new: bool,
        messages: Vec<MemoryEntry>,
    ) -> Result<()> {
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
                        let MemoryEntry {
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
                            MemoryContent::File(file)=>Some(MessageItem {
                                id: id as i32,
                                is_me: role != ChatRole::Assistant,
                                kind: "text".into(),
                                text: format!("{:?}",file).into(),
                                image: Image::default(),
                                time: time.format("%Y-%m-%d %H:%M:%S").to_string().into(),
                            })
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
            .context("绘图 bus_stats_chart 失败")
    }
}
