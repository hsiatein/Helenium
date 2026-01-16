use std::{collections::HashMap, time::Duration};

use anyhow::{Context, Result};
use async_trait::async_trait;
use heleny_bus::endpoint::Endpoint;
use heleny_proto::{CanRequestConsent, ChatRole, FS_SERVICE, HelenyFile, HelenyTool, HelenyToolFactory, MEMORY_SERVICE, get_tool_arg};
use heleny_service::{FsServiceMessage, MemoryServiceMessage};
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::{sync::oneshot};
use uuid::Uuid;
use rand::Rng;


#[derive(Debug)]
pub struct ComfyuiTool {
    endpoint:Endpoint,
    comfyui_url:String,
    comfyui_auth:Option<(String,String)>,
    base_prompt:String,
    api_key:String,
}

impl ComfyuiTool {
    pub async fn new(endpoint:Endpoint, comfyui_url:String, comfyui_auth:Option<String>, base_prompt:String, api_key:String)->Result<Self> {
        let client=Client::new();
        
        let comfyui_url=comfyui_url.trim_end_matches("/").to_string();
        let comfyui_auth = match comfyui_auth {
            Some(auth)=>{
                let split:Vec<&str>=auth.split(":").collect();
                let user=split.get(0).context("获取用户名失败")?.to_string();
                let password=split.get(1).context("获取密码失败")?.to_string();
                let _=client.get(format!("{comfyui_url}/system_stats")).basic_auth(&user, Some(&password)).send().await?.error_for_status()?;
                Some((user,password))
            }
            None=>{
                let _=client.get(format!("{comfyui_url}/system_stats")).send().await?.error_for_status()?;
                None
            }
        };
        
        Ok(Self { endpoint, comfyui_url, comfyui_auth, base_prompt, api_key })
    }

    fn auth(&self,rb:RequestBuilder)->RequestBuilder {
        match &self.comfyui_auth {
            Some((user,password))=>{
                rb.basic_auth(user, Some(password))
            }
            None=> rb,
        }
    }
}

#[async_trait]
impl HelenyTool for ComfyuiTool {
    async fn invoke(
        &mut self,
        command: String,
        args: HashMap<String, Value>,
        _request: Box<&dyn CanRequestConsent>,
    ) -> Result<String>{
        if command!="generate" {
            return Err(anyhow::anyhow!("未知 Command"));
        }
        let input_prompt=ComfyuiPrompt::new(args);
        let prompt=input_prompt.replace(&self.base_prompt)?;
        let client=Client::new();
        let resp=self.auth(client.post(self.comfyui_url.clone()+"/prompt")).json(&json!({
            "prompt": prompt,
            "extra_data": {
                "api_key_comfy_org": self.api_key
            }
        })).send()
        .await.context("发送请求失败")?;
        let resp:HashMap<String,Value>=serde_json::from_str(&resp.text().await?).context("解析响应失败")?;
        let prompt_id=resp.get("prompt_id").context("获取 prompt_id 失败")?.as_str().context("获取 prompt_id str 失败")?;
        for _ in 0..3600 {
            let body=self.auth(client.get(self.comfyui_url.clone()+"/history/"+prompt_id)).send().await?.error_for_status()?.text().await?;
            if body.len()>10 {
                println!("{}",body);
                break;
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        let download_url =self.comfyui_url.clone()+"/view?filename="+&input_prompt.file_name+".png&type=output";
        let bytes:Vec<u8> = self.auth(client.get(download_url))
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?.into();
        let (tx,rx)=oneshot::channel();
        self.endpoint.send(FS_SERVICE, FsServiceMessage::TempFile { file: HelenyFile::Image(bytes), file_ext: "png".into(), feedback: tx }).await?;
        let path=rx.await?;
        self.endpoint
            .send(MEMORY_SERVICE, MemoryServiceMessage::Post { role: ChatRole::Assistant, content: path.into() })
            .await?;
        Ok("图片生成完成".into())
    }
}

#[async_trait]
impl HelenyToolFactory for ComfyuiTool {
    fn name(&self) -> String{
        "comfyui".into()
    }
    async fn create(&mut self) -> Result<Box<dyn HelenyTool>>{
        let tool=ComfyuiTool { endpoint: self.endpoint.create_sender_endpoint(), comfyui_url: self.comfyui_url.clone(), comfyui_auth: self.comfyui_auth.clone(), base_prompt: self.base_prompt.clone(), api_key: self.api_key.clone() };
        Ok(Box::new(tool))
    }
}


#[derive(Debug,Serialize,Deserialize)]
pub struct ComfyuiPrompt {
    pub appearance:String,
    pub clothes:String,
    pub action:String,
    pub style:String,
    pub negative:String,
    pub extra:String,
    pub seeds:[i64;6],
    pub file_name:String,
}

impl ComfyuiPrompt {
    pub fn replace(&self,prompt:&String)->Result<Value> {
        let mut prompt=prompt.replace("<额外内容>", &self.extra);
        prompt=prompt.replace("<外貌>", &self.appearance);
        prompt=prompt.replace("<服装>", &self.clothes);
        prompt=prompt.replace("<动作>", &self.action);
        prompt=prompt.replace("<风格>", &self.style);
        prompt=prompt.replace("<负面词条>", &self.negative);
        prompt=prompt.replace("<文件名>", &self.file_name);
        prompt=prompt.replace("\"<seed_face>\"", &self.seeds[0].to_string());
        prompt=prompt.replace("\"<seed_sampler>\"", &self.seeds[1].to_string());
        prompt=prompt.replace("\"<seed_upsampler>\"", &self.seeds[2].to_string());
        prompt=prompt.replace("\"<seed_nipples>\"", &self.seeds[3].to_string());
        prompt=prompt.replace("\"<seed_hands>\"", &self.seeds[4].to_string());
        prompt=prompt.replace("\"<seed_eyes>\"", &self.seeds[5].to_string());
        Ok(serde_json::from_str(&prompt).context("转为 json prompt 失败")?)
    }

    pub fn new(mut args:HashMap<String, Value>)->Self {
        let mut prompt=Self::default();
        // if let Ok(appearance)=get_tool_arg::<String>(&mut args, "appearance"){
        //     prompt.appearance=appearance;
        // };
        if let Ok(clothes)=get_tool_arg::<String>(&mut args, "clothes"){
            prompt.clothes=clothes;
        };
        if let Ok(action)=get_tool_arg::<String>(&mut args, "action"){
            prompt.action=action;
        };
        // if let Ok(style)=get_tool_arg::<String>(&mut args, "style"){
        //     prompt.style=style;
        // };
        // if let Ok(negative)=get_tool_arg::<String>(&mut args, "negative"){
        //     prompt.negative=negative;
        // };
        if let Ok(extra)=get_tool_arg::<String>(&mut args, "extra"){
            prompt.extra=extra;
        };
        prompt
    }
}



impl Default for ComfyuiPrompt {
    fn default() -> Self {
        let seeds=[rand::thread_rng().gen_range(0..i64::MAX),rand::thread_rng().gen_range(0..i64::MAX),rand::thread_rng().gen_range(0..i64::MAX),rand::thread_rng().gen_range(0..i64::MAX),rand::thread_rng().gen_range(0..i64::MAX),rand::thread_rng().gen_range(0..i64::MAX)];
        Self { appearance: default_appearance(), clothes: default_clothes(), action: default_action(), style: default_style(), negative: default_negative(), extra: "".into(),seeds,file_name: Uuid::new_v4().to_string() }
    }
}

fn default_appearance()->String {
    "Simple eyes, blue eyes, silver hair, (hair between eyes:0.8), (messy hair:0.8), short sidelocks, detailed eyes, (short hair:0.8), (medium hair:1.2), straight_hair, small breasts, young girl, refreshing face, smile, white_skin, (low_twintails:1.2), (short_twintails:0.5),".into()
}

fn default_clothes()->String {
    "black leotard, animal collar, fake animal ears, sleeveless, leotard, jewelry,rabbit ears, suit, sweat, suit, white background, tailcoat, upper body, collarbone,".into()
}

fn default_action()->String {
    "looking_at_viewer,".into()
}

fn default_style()->String {
    "girl, solo, female focus,dynamic angle,depth of field,high contrast,colorful,detailed light,light leaks,beautiful detailed glow,best shadow,shiny skin,ray tracing,detailed light,sunlight,shine on girl's body,cinematic lighting,oiled,(artist:mika pikazo:0.5),(artist:ciloranko:0.5),(artist:kazutake hazano:0.5),(artist:kedama milk:0.5),(artist:ask_(askzy):0.5),(artist:wanke:0.5),(artist:fujiyama:0.5),year 2024,".into()
}

fn default_negative()->String {
    "colored inner hair, gradient hair color, fluffy_hair, mature, high twintails, high ponytail, high twin buns, high pigtails,".into()
}


#[cfg(test)]
mod tests{
    use tokio::fs;

    use super::*;
    #[tokio::test]
    async fn test_comfyui()->Result<()>{
        let mut prompt=fs::read_to_string("../assets/tool-resource/obs赫蕾妮-通用.json").await?;
        println!("{prompt}");
        let input=ComfyuiPrompt::default();
        prompt=prompt.replace("<额外内容>", &input.extra);
        prompt=prompt.replace("<外貌>", &input.appearance);
        prompt=prompt.replace("<服装>", &input.clothes);
        prompt=prompt.replace("<动作>", &input.action);
        prompt=prompt.replace("<风格>", &input.style);
        prompt=prompt.replace("<负面词条>", &input.negative);
        fs::write("../.temp/test.json", prompt).await?;
        Ok(())
    }
    // #[tokio::test]
    // async fn test_comfyui2()->Result<()>{
    //     dotenvy::dotenv().ok();
    //     let base_prompt=fs::read_to_string("../assets/tool-resource/obs赫蕾妮-通用.json").await?;
    //     let api_key= std::env::var("COMFYUI_API_KEY")?;
    //     let mut comfyui=ComfyuiTool::new("http://127.0.0.1:8188".into(), base_prompt, api_key).await?;
    //     // let input=ComfyuiPrompt::default();
    //     comfyui.invoke("generate".into(), HashMap::new(), Box::new(& TestCanRequestConsent::new())).await?;
    //     Ok(())
    // }
    #[tokio::test]
    async fn test_history()->Result<()>{
        let body=reqwest::get("http://127.0.0.1:8188/history/700994f1-bfe5-47eb-8f6b-b3fff11dfcf71").await?.text().await?;
        println!("{}, {}, {}",body,body.len(),body=="{}");
        let url ="http://127.0.0.1:8188/view?filename=53cfa3b7-163a-48fc-9635-2a36f3977464.png&type=output";
        let client=Client::new();
        let bytes = client
            .get(url)
            .send()
            .await?
            .error_for_status()?   // 非 200 直接报错
            .bytes()
            .await?;
        let bytes:Vec<u8>=bytes.into();
        fs::write("../.temp/test.png", bytes).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_auth()->Result<()>{
        dotenvy::dotenv().ok();
        Ok(())
    }
}