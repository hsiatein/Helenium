use crate::FrontendHandler;
use crate::ScheduleItem;
use anyhow::Context;
use anyhow::Result;
use heleny_proto::ScheduledTask;
use slint::ModelRc;
use slint::SharedString;
use slint::VecModel;
use std::collections::HashMap;
use uuid::Uuid;

impl FrontendHandler {
    pub async fn handle_schedules(&self, schedules: HashMap<Uuid, ScheduledTask>) -> Result<()> {
        let _ = self
            .ui_weak
            .upgrade_in_event_loop(move |ui| {
                let schedules: Vec<ScheduleItem> = schedules
                    .into_iter()
                    .map(|(id, task)| {
                        let ScheduledTask {
                            description,
                            triggers,
                            offset: _,
                            next_trigger,
                        } = task;
                        let next_trigger = if let Some(next_trigger) = next_trigger {
                            next_trigger.format("%Y-%m-%d %H:%M:%S").to_string()
                        } else {
                            format!("没有下次运行")
                        };
                        let triggers: Vec<SharedString> = triggers
                            .into_iter()
                            .map(|trigger| SharedString::from(trigger.to_string()))
                            .collect();
                        ScheduleItem {
                            id: SharedString::from(id.to_string()),
                            description: SharedString::from(description),
                            next_trigger: SharedString::from(next_trigger),
                            triggers: ModelRc::new(VecModel::from(triggers)),
                        }
                    })
                    .collect();
                ui.set_schedules(ModelRc::new(VecModel::from(schedules)));
            })
            .context("日程更新显示失败");
        Ok(())
    }
}
