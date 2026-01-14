use std::collections::HashMap;

use anyhow::{Context, Result};
use async_trait::async_trait;
use heleny_proto::{CanRequestConsent, HelenyTool, HelenyToolFactory, get_tool_arg};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;
use rand::Rng;

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
        if let Ok(appearance)=get_tool_arg::<String>(&mut args, "appearance"){
            prompt.appearance=appearance;
        };
        if let Ok(clothes)=get_tool_arg::<String>(&mut args, "clothes"){
            prompt.clothes=clothes;
        };
        if let Ok(action)=get_tool_arg::<String>(&mut args, "action"){
            prompt.action=action;
        };
        if let Ok(style)=get_tool_arg::<String>(&mut args, "style"){
            prompt.style=style;
        };
        if let Ok(negative)=get_tool_arg::<String>(&mut args, "negative"){
            prompt.negative=negative;
        };
        if let Ok(extra)=get_tool_arg::<String>(&mut args, "extra"){
            prompt.extra=extra;
        };
        prompt
    }
}

#[derive(Debug,Clone)]
pub struct ComfyuiTool {
    comfyui_url:String,
    base_prompt:String,
    api_key:String,
}

impl ComfyuiTool {
    pub async fn new(comfyui_url:String, base_prompt:String, api_key:String)->Result<Self> {
        let comfyui_url=comfyui_url.trim_end_matches("/").to_string();
        let _=reqwest::get(format!("{comfyui_url}/system_stats")).await?.text().await?;
        Ok(Self { comfyui_url, base_prompt, api_key })
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
        let resp=client.post(self.comfyui_url.clone()+"/prompt").json(&json!({
            "prompt": prompt,
            "extra_data": {
                "api_key_comfy_org": self.api_key
            }
        })).send()
        .await?;
        let status = resp.status();
        let body = resp.text().await?;
        println!("status: {}", status);
        println!("body: {}", body);
        Ok("绘图完成".into())
    }
}

#[async_trait]
impl HelenyToolFactory for ComfyuiTool {
    fn name(&self) -> String{
        "comfyui".into()
    }
    async fn create(&mut self) -> Result<Box<dyn HelenyTool>>{
        let tool=ComfyuiTool { comfyui_url: self.comfyui_url.clone(), base_prompt: self.base_prompt.clone(), api_key: self.api_key.clone() };
        Ok(Box::new(tool))
    }
}

#[cfg(test)]
mod tests{
    use heleny_proto::TestCanRequestConsent;
    use tokio::fs;

    use super::*;
    #[tokio::test]
    async fn test_comfyui()->Result<()>{
        let body=reqwest::get("http://127.0.0.1:8188/system_stats").await?.text().await?;
        println!("{body}");
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
    #[tokio::test]
    async fn test_comfyui2()->Result<()>{
        dotenvy::dotenv().ok();
        let base_prompt=fs::read_to_string("../assets/tool-resource/obs赫蕾妮-通用.json").await?;
        let api_key= std::env::var("COMFYUI_API_KEY")?;
        let mut comfyui=ComfyuiTool::new("http://127.0.0.1:8188".into(), base_prompt, api_key).await?;
        // let input=ComfyuiPrompt::default();
        comfyui.invoke("generate".into(), HashMap::new(), Box::new(& TestCanRequestConsent::new())).await?;
        Ok(())
    }
    #[tokio::test]
    async fn test_history()->Result<()>{
        let body=reqwest::get("http://127.0.0.1:8188/history/700994f1-bfe5-47eb-8f6b-b3fff11dfcf7").await?.text().await?;
        println!("{body}");
        Ok(())
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
    "from side, girl, solo, female focus,dynamic angle,depth of field,high contrast,colorful,detailed light,light leaks,beautiful detailed glow,best shadow,shiny skin,ray tracing,detailed light,sunlight,shine on girl's body,cinematic lighting,oiled,(artist:mika pikazo:0.5),(artist:ciloranko:0.5),(artist:kazutake hazano:0.5),(artist:kedama milk:0.5),(artist:ask_(askzy):0.5),(artist:wanke:0.5),(artist:fujiyama:0.5),year 2024,".into()
}

fn default_negative()->String {
    "colored inner hair, gradient hair color, fluffy_hair, mature, hand, hands, high twintails, high ponytail, high twin buns, high pigtails,".into()
}