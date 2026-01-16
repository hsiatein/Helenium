use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        return;
    }
    let name = &args[1];
    let project_dir_str = format!("service-{}", name);
    std::process::Command::new("cargo")
        .arg("new")
        .arg(&project_dir_str)
        .spawn()
        .expect("cargo new 失败");

    let current_dir = std::env::current_dir().expect("读取当前文件夹失败");
    println!("{:?}", current_dir);
    let project_dir = current_dir.join(&project_dir_str);
    let manifest_path = project_dir.join("Cargo.toml");
    println!("{:?}", manifest_path);
    sleep(Duration::from_millis(500));

    let manifest = std::fs::read_to_string(&manifest_path).expect("读取失败");
    let manifest = manifest.replace(&project_dir_str, &format!("service_{}", name));
    let manifest = manifest
        + r#"heleny_service = {path = "../heleny-service"}
heleny_proto = {path = "../heleny-proto"}
heleny_macros = { path = "../heleny-macros" }
heleny_bus = { path = "../heleny-bus" }
async-trait = { workspace = true }
inventory = { workspace = true }
anyhow = {workspace = true}
tracing = {workspace = true}
tokio = {workspace = true}
serde_json = {workspace = true}
serde = {workspace = true}
"#;
    std::fs::write(manifest_path, manifest).expect("写入失败");

    std::fs::remove_file(project_dir.join("src").join("main.rs")).expect("删除main失败");

    let code = r#"use tokio::time::Instant;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_service::Service;
use heleny_service::{PATTERN}ServiceMessage;
use heleny_proto::{AnyMessage, ServiceRole};
use async_trait::async_trait;
use anyhow::Result;
use heleny_proto::Resource;


#[base_service(deps=[])]
pub struct {PATTERN}Service{
    endpoint:Endpoint,
}

#[derive(Debug)]
enum _WorkerMessage{
    
}

#[async_trait]
impl Service for {PATTERN}Service {
    type MessageType= {PATTERN}ServiceMessage;
    async fn new(endpoint: Endpoint) -> Result<Box<Self>>{
        // 实例化
        let instance=Self {
            endpoint,
        };
        Ok(Box::new(instance))
    }
    async fn handle(
        &mut self,
        _name: String,
        _role: ServiceRole,
        _msg: {PATTERN}ServiceMessage,
    ) -> Result<()>{
        Ok(())
    }
    async fn stop(&mut self){

    }
    async fn handle_sub_endpoint(&mut self, _msg: Box<dyn AnyMessage>) -> Result<()>{
        Ok(())
    }
    async fn handle_tick(&mut self, _tick:Instant) -> Result<()>{
        Ok(())
    }
    async fn handle_resource(&mut self, _resource: Resource) -> Result<()> {
        Ok(())
    }
}

impl {PATTERN}Service {
    
}
"#;
    let code = code.replace("{pattern}", &name);
    let code = code.replace("{PATTERN}", &capitalize(&name));
    std::fs::write(project_dir.join("src").join("lib.rs"), code).expect("写入lib.rs失败");

    let message = r#"#[derive(Debug)]
pub enum {PATTERN}ServiceMessage {

}"#;
    let message = message.replace("{PATTERN}", &capitalize(&name));
    std::fs::write(
        PathBuf::from("heleny-service")
            .join("src")
            .join("messages")
            .join(format!("{}_service_message.rs", name)),
        message,
    )
    .expect("写入message枚举失败");

    let path = &PathBuf::from("heleny-service")
        .join("src")
        .join("messages.rs");
    let message_lib = std::fs::read_to_string(path).expect("读取失败");
    let message_lib = message_lib
        + "\nmod "
        + name
        + "_service_message;\npub use "
        + name
        + "_service_message::*;";
    std::fs::write(path, message_lib).expect("写入mod声明失败");

    let manifest_path = current_dir.join("heleny-kernel").join("Cargo.toml");
    let manifest =
        std::fs::read_to_string(&manifest_path).expect("读取heleny-kernel cargo.toml失败");
    let line = format!("service_{} ={{ path = \"../service-{}\"}}", &name, &name);
    let manifest = manifest + "\n" + line.as_str();
    std::fs::write(manifest_path, manifest).expect("写入库依赖声明失败");

    let kernel_path = current_dir.join("heleny-kernel").join("src").join("lib.rs");
    let lib = std::fs::read_to_string(&kernel_path).expect("读取heleny-kernel lib.rs失败");
    let line = format!("extern crate service_{};", &name);
    let lib = lib + "\n" + line.as_str();
    std::fs::write(&kernel_path, lib).expect("写入库依赖声明失败");

    println!("成功新建 {}Service", &capitalize(&name));
}
