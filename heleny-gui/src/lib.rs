use anyhow::Context;
use anyhow::Result;
use heleny_proto::ConsentRequestionFE;
use heleny_proto::FrontendCommand;
use heleny_proto::FrontendMessage;
use heleny_proto::UserDecision;
use slint::Model;
use slint::ModelRc;
use slint::Weak;
use tokio::sync::mpsc;
use tracing::debug;
mod handle_ws;
pub use handle_ws::*;
mod set_callback;
pub use set_callback::*;
mod terminal;

slint::include_modules!();
mod handle_resource;
mod init_resource;
pub use init_resource::*;

pub struct FrontendHandler {
    writer: mpsc::Sender<FrontendCommand>,
    ui_weak: Weak<AppWindow>,
}

impl FrontendHandler {
    pub fn new(writer: mpsc::Sender<FrontendCommand>, ui_weak: Weak<AppWindow>) -> Self {
        Self { writer, ui_weak }
    }
    pub async fn handle_frontend_message(&self, msg: FrontendMessage) -> Result<()> {
        match msg {
            FrontendMessage::UpdateResource(resource) => {
                self.handle_resource(resource.payload).await?
            }
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
