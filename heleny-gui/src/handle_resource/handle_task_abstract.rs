use crate::FrontendHandler;
use crate::TaskItem;
use anyhow::Context;
use anyhow::Result;
use heleny_proto::TaskAbstract;
use slint::Model;
use slint::ModelRc;
use slint::SharedString;
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

impl FrontendHandler {
    pub async fn handle_task_abstract(&self, task_abstracts: Vec<TaskAbstract>) -> Result<()> {
        self.ui_weak
            .upgrade_in_event_loop(|ui| {
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
                                    logs: ModelRc::new(slint::VecModel::from(
                                        Vec::<SharedString>::new(),
                                    )),
                                },
                            }
                        },
                    )
                    .collect();
                ui.set_tasks(ModelRc::new(slint::VecModel::from(new_tasks)));
            })
            .context("任务摘要显示失败")
    }
}
