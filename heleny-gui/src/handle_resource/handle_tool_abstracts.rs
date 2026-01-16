use crate::CommandItem;
use crate::FrontendHandler;
use crate::ToolAbstractItem;
use anyhow::Context;
use anyhow::Result;
use heleny_proto::ToolAbstract;
use slint::Model;
use slint::ModelRc;
use slint::SharedString;
use slint::VecModel;
use std::collections::HashMap;

impl FrontendHandler {
    pub async fn handle_tool_abstracts(&self, abstracts: Vec<ToolAbstract>) -> Result<()> {
        self.ui_weak
            .upgrade_in_event_loop(|ui| {
                let mut new_abstracts: HashMap<String, ToolAbstractItem> = ui
                    .get_tool_abstracts()
                    .iter()
                    .map(|abs| (abs.name.to_string(), abs))
                    .collect();
                for abs in abstracts {
                    let ToolAbstract {
                        name,
                        description,
                        commands,
                        available,
                        enable,
                    } = abs;
                    let commands: Vec<CommandItem> = commands
                        .into_iter()
                        .map(|(name, des)| CommandItem {
                            description: SharedString::from(des),
                            name: SharedString::from(name),
                        })
                        .collect();
                    match new_abstracts.get_mut(&name) {
                        Some(abs) => {
                            abs.available = available;
                            abs.commands = ModelRc::new(VecModel::from(commands));
                            abs.description = SharedString::from(description);
                            abs.enable = enable;
                        }
                        None => {
                            let item = ToolAbstractItem {
                                available,
                                commands: ModelRc::new(VecModel::from(commands)),
                                description: SharedString::from(description),
                                expanded: false,
                                name: SharedString::from(name.clone()),
                                desc_expanded: false,
                                enable: enable,
                            };
                            new_abstracts.insert(name, item);
                        }
                    }
                }
                let mut new_abstracts: Vec<ToolAbstractItem> =
                    new_abstracts.into_values().collect();
                new_abstracts.sort_by(|a, b| {
                    b.available
                        .cmp(&a.available)
                        .then_with(|| a.name.cmp(&b.name))
                });
                ui.set_tool_abstracts(ModelRc::new(VecModel::from(new_abstracts)));
            })
            .context("工具摘要显示失败")
    }
}
