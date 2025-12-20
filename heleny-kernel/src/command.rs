use heleny_proto::message::AnyMessage;
use heleny_service::ServiceHandle;
use anyhow::Result;

pub enum KernelCommand{
    Shutdown,
    AddService(ServiceHandle),
}

impl KernelCommand {
    pub fn downcast(msg: Box<dyn AnyMessage>) -> Result<Box<KernelCommand>> {
        msg.as_any().downcast::<KernelCommand>()
            .map_err(|_| anyhow::anyhow!(
                "消息类型转换失败：期望类型为 KernelCommand, 但收到的是其他类型"))
    }
}