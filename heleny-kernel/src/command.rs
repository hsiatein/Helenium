use anyhow::Result;
use heleny_bus::endpoint::Endpoint;
use heleny_proto::message::AnyMessage;
use tokio::sync::oneshot;

use heleny_proto::kernel_message::KernelMessage;

#[derive(Debug)]
pub enum AdminCommand {
    NewEndpoint(&'static str, oneshot::Sender<Endpoint>),
    Shutdown(ShutdownStage),
}

#[derive(Debug)]
pub enum ShutdownStage {
    Start,
    StopAllService,
    StopKernel,
}

pub fn downcast(msg: Box<dyn AnyMessage>) -> Result<Result<Box<AdminCommand>, Box<KernelMessage>>> {
    let command = match msg.as_any().downcast::<AdminCommand>() {
        Ok(command) => return Ok(Ok(command)),
        Err(command) => command,
    };
    match command.downcast::<KernelMessage>() {
        Ok(command) => Ok(Err(command)),
        Err(_) => Err(anyhow::anyhow!("不是 KernelCommand 也不是 AdminCommand")),
    }
}
