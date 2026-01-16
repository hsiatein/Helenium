use anyhow::Result;
use heleny_proto::FrontendCommand;
use tokio::sync::mpsc;

pub async fn init_resource(write_tx: &mpsc::Sender<FrontendCommand>) -> Result<()> {
    write_tx
        .send(FrontendCommand::GetHistory(1000000000))
        .await?;
    write_tx.send(FrontendCommand::GetHealth).await?;
    write_tx
        .send(FrontendCommand::GetConsentRequestions)
        .await?;
    write_tx.send(FrontendCommand::GetSchedules).await?;
    write_tx.send(FrontendCommand::GetToolAbstrats).await?;
    Ok(())
}
