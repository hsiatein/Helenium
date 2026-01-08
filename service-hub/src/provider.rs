use std::collections::HashSet;

use anyhow::Context;
use anyhow::Result;
use heleny_bus::endpoint::Endpoint;
use heleny_proto::Resource;
use heleny_proto::ResourcePayload;
use heleny_service::CommonMessage;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tracing::warn;

pub struct Provider {
    // 提供者的名字
    pub name: String,
    pub handle: JoinHandle<()>,
    tx: mpsc::Sender<Command>,
}

enum Command {
    Add(String),
    Delete(String),
    Get(oneshot::Sender<ResourcePayload>),
}

impl Provider {
    pub fn new(
        name: String,
        resource_name: String,
        endpoint: Endpoint,
        receiver: watch::Receiver<ResourcePayload>,
        subscribers: HashSet<String>,
    ) -> Result<Provider> {
        let (tx, mut rx) = mpsc::channel(32);
        let mut worker = ProviderWorker::new(endpoint, receiver, resource_name, subscribers);
        let handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = worker.receiver.changed() => {
                        if let Err(e)=worker.handle().await{
                            warn!("{} 处理时出错: {}",worker.resource_name,e)
                        };
                    }

                    Some(command) = rx.recv() =>{
                        match command {
                            Command::Add(name)=>{
                                worker.subscribers.insert(name);
                            }
                            Command::Delete(name)=>{
                                worker.subscribers.remove(&name);
                            }
                            Command::Get(feedback)=>{
                                let resource=worker.receiver.borrow().to_owned();
                                let _=feedback.send(resource);
                            }
                        };
                    }

                }
            }
        });
        let instance = Self { name, handle, tx };
        Ok(instance)
    }

    pub async fn subscribe(&self, name: String) -> Result<()> {
        self.tx.send(Command::Add(name)).await.context("发送失败")
    }

    pub async fn unsubscribe(&self, name: String) -> Result<()> {
        self.tx
            .send(Command::Delete(name))
            .await
            .context("发送失败")
    }
    pub async fn get(&self, feedback: oneshot::Sender<ResourcePayload>) -> Result<()> {
        self.tx
            .send(Command::Get(feedback))
            .await
            .context("发送失败")
    }
}

struct ProviderWorker {
    endpoint: Endpoint,
    pub receiver: watch::Receiver<ResourcePayload>,
    pub subscribers: HashSet<String>,
    resource_name: String,
}

impl ProviderWorker {
    pub fn new(
        endpoint: Endpoint,
        receiver: watch::Receiver<ResourcePayload>,
        resource_name: String,
        subscribers: HashSet<String>,
    ) -> Self {
        Self {
            endpoint,
            receiver,
            subscribers,
            resource_name,
        }
    }

    async fn handle(&mut self) -> Result<()> {
        let resource = self.receiver.borrow_and_update().to_owned();
        for subscriber in &self.subscribers {
            if let Err(e) = self
                .endpoint
                .send(
                    subscriber,
                    CommonMessage::Resource(Resource {
                        name: self.resource_name.clone(),
                        payload: resource.clone(),
                    }),
                )
                .await
            {
                warn!("{} 发送给 {} 失败: {}", self.resource_name, subscriber, e);
            }
        }
        Ok(())
    }
}
