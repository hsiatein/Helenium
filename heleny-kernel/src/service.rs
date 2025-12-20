use std::{collections::{HashMap, HashSet}, time::Duration};

use anyhow::{Result};
use heleny_bus::Endpoint;
use heleny_service::{Service, ServiceFactory};
use async_trait::async_trait;
use heleny_macros::{base_service};
use inventory;
use tokio::{sync::oneshot, time::timeout};

use crate::{command::KernelCommand, health::{HealthStatus, KernelHealth}};

pub enum KernelMessage {
    Shutdown,
    Init,
}

#[base_service(deps=[])]
pub struct KernelService {
    endpoint: heleny_bus::Endpoint,
    service_factories: Vec<&'static ServiceFactory>,
    order: Option<Result<Vec<&'static str>>>,
    health: KernelHealth,
}

#[async_trait]
impl Service for KernelService {
    type MessageType = KernelMessage;
    fn new(endpoint:heleny_bus::Endpoint) -> Box<Self> {
        let service_factories=inventory::iter::<ServiceFactory>.into_iter().filter(|factory|{
            factory.name!="KernelService"
        }).collect();
        Box::new(Self { endpoint, service_factories, order: None, health: KernelHealth::new() })
    }
    async fn handle(&mut self, msg: Box<KernelMessage>) -> anyhow::Result<()> {
        match msg.as_ref() {
            KernelMessage::Shutdown => {
                println!("KernelService 收到关闭指令，正在关闭...");
                let _=self.endpoint.send("Kernel",Box::new(KernelCommand::Shutdown)).await;
            },
            KernelMessage::Init => {
                self.init_service().await;
            }
        }
        Ok(())
    }
}

impl KernelService {
    /// 从内核获取 Endpoint
    async fn get_endpoint(&self, name: &'static str) -> Result<Endpoint> {
        let (tx,rx)=oneshot::channel();
        let _=self.endpoint.send("Kernel", Box::new(KernelCommand::NewEndpoint(name, tx))).await;
        match timeout(Duration::from_secs(5), rx).await {
            Ok(Ok(endpoint)) => Ok(endpoint),
            Ok(Err(e)) => Err(anyhow::anyhow!("获取 Endpoint 时错误: {}",e)),
            Err(e) => Err(anyhow::anyhow!("超时: {}",e))
        }
    }

    /// 初始化各个服务
    async fn init_service(&mut self) {
        let order=match self.get_order() {
            Ok(order) => order,
            Err(e) => {
                eprintln!("无法获取Order, 初始化失败: {}",e);
                self.health.services=self.health.services.iter().map(|(name,_)| (*name,HealthStatus::Dead)).collect();
                return ;
            }
        };
        for name in order{
            let factory=match self.service_factories.iter().find(|f| f.name==name) {
                Some(factory) => factory,
                None => {
                    self.health.services.insert(name, HealthStatus::Dead);
                    continue;
                }
            };
            if !self.is_deps_ready(&factory.deps) {
                eprintln!("依赖未准备完成, 跳过初始化");
                self.health.services.insert(name, HealthStatus::Dead);
                continue;
            }
            let endpoint=match self.get_endpoint(name).await {
                Ok(endpoint) => endpoint,
                Err(e) => {
                    eprintln!("无法获取 Endpoint: {}",e);
                    self.health.services.insert(name, HealthStatus::Dead);
                    continue;
                }
            };
            let handle=match (factory.launch)(endpoint){
                Ok(handle) =>handle,
                Err(e) => {
                    eprintln!("初始化服务失败: {}",e);
                    self.health.services.insert(name, HealthStatus::Dead);
                    continue;
                }
            };
            let _=self.endpoint.send("Kernel", Box::new(KernelCommand::AddService(handle))).await;
            self.health.services.insert(name, HealthStatus::Healthy);
            println!("初始化服务成功: {}",name);

        }
    }

    /// 获取初始化顺序
    fn get_order(&mut self)->Result<Vec<&'static str>>{
        if self.order.is_none() {
            let dag_map=self.service_factories.iter().map(|f|{
                (f.name,f.deps.iter().copied().collect::<HashSet<&'static str>>())
            }).collect::<HashMap<&'static str,HashSet<&'static str>>>();
            let order=cal_order(dag_map);
            self.order=Some(order);
        }
        let result=&self.order;
        match result {
            None => Err(anyhow::anyhow!("计算初始化顺序失败")),
            Some(Ok(order)) => {
                Ok(order.clone())
            },
            Some(Err(e)) => {
                Err(anyhow::anyhow!("{}",e))
            },
        }
    }

    /// 检测前置服务是否准备好
    fn is_deps_ready(&self,deps:&Vec<&'static str>)->bool {
        deps.iter().all(|&name| self.health.services.get(name) == Some(&HealthStatus::Healthy))
    }
}

fn cal_order(mut dag_map: HashMap<&'static str,HashSet<&'static str>>)->Result<Vec<&'static str>>{
    let mut order=Vec::new();
    let mut last_len=0;
    while last_len!=dag_map.len() {
        last_len=dag_map.len();
        let (new,remain):(HashMap<&'static str,HashSet<&'static str>>,HashMap<&'static str,HashSet<&'static str>>)=dag_map.into_iter().partition(|(_,deps)| deps.len()==0);
        let new=new.keys().copied().collect::<HashSet<&'static str>>();
        dag_map=remain.into_iter().map(|(k,deps)| (k,&deps-&new)).collect();
        order.extend(new);
    }
    if dag_map.len()==0 {
        Ok(order)
    }
    else {
        Err(anyhow::anyhow!("有循环依赖或未知依赖 {:?}",dag_map.keys()))
    }
}

#[cfg(test)]
mod test_order;