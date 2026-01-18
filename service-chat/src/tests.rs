use std::env;

use anyhow::Result;
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::chat::ChatCompletionRequestMessage;
use async_openai::types::chat::ChatCompletionRequestSystemMessageArgs;
use async_openai::types::chat::ChatCompletionRequestUserMessageArgs;
use async_openai::types::chat::ChatCompletionResponseStream;
use async_openai::types::chat::CreateChatCompletionRequestArgs;
use async_openai::types::chat::ResponseFormat;
use async_openai::types::chat::ResponseFormatJsonSchema;
use async_openai::types::embeddings::CreateEmbeddingRequest;
use async_openai::types::embeddings::EmbeddingInput;
use async_openai::types::responses::CreateResponseArgs;
use gemini_rust::Gemini;
use genai::chat::ChatOptions;
use genai::embed::EmbedOptions;
use tokio_stream::StreamExt;

#[tokio::test]
async fn test_api() {
    // Create client
    dotenvy::dotenv().ok();
    let api_key = std::env::var("QWEN_API_KEY").expect("无 API KEY");
    let config = OpenAIConfig::new()
        .with_api_base("https://dashscope.aliyuncs.com/compatible-mode/v1")
        .with_api_key(api_key);
    let client = Client::with_config(config);
    // Create request using builder pattern
    // Every request struct has companion builder struct with same name + Args suffix
    let request = CreateResponseArgs::default()
        .model("qwen-max")
        .input("你是谁")
        .max_output_tokens(512u32)
        .build()
        .expect("构造请求失败");

    // Call API
    let response: async_openai::types::responses::Response = client
        .responses() // Get the API "group" (responses, images, etc.) from the client
        .create(request) // Make the API call in that "group"
        .await
        .expect("获取回复失败");

    println!("{:?}", response.output_text());
}

#[tokio::test]
async fn test_prompt() {
    // Create client
    dotenvy::dotenv().ok();
    let api_key = std::env::var("GROK_API_KEY").expect("无 API KEY");
    let config = OpenAIConfig::new()
        .with_api_base("https://api.x.ai/v1")
        .with_api_key(api_key);
    let client = Client::with_config(config);
    let system_msg = ChatCompletionRequestSystemMessageArgs::default()
        .content("你是一个助手名字叫 Heleny 。")
        .build()
        .expect("系统提示词失败");
    let user_msg = ChatCompletionRequestUserMessageArgs::default()
        .content("你是谁")
        .build()
        .expect("用户提示词失败");
    // Create request using builder pattern
    // Every request struct has companion builder struct with same name + Args suffix
    let request = CreateChatCompletionRequestArgs::default()
        .model("grok-4-1-fast-non-reasoning") // 建议确认模型名称是否准确
        .messages([
            system_msg.into(), // 系统提示词通常排在第一条
            user_msg.into(),
        ])
        .n(1)
        .build()
        .expect("构造请求失败");
    // Call API
    let response = client
        .chat() // Get the API "group" (responses, images, etc.) from the client
        .create(request) // Make the API call in that "group"
        .await
        .expect("获取回复失败");
    print!("{:?}", response.choices[0].message.content);
}

#[tokio::test]
async fn test_stream() {
    // Create client
    dotenvy::dotenv().ok();
    let api_key = std::env::var("QWEN_API_KEY").expect("无 API KEY");
    let config = OpenAIConfig::new()
        .with_api_base("https://dashscope.aliyuncs.com/compatible-mode/v1")
        .with_api_key(api_key);
    let client = Client::with_config(config);
    let system_msg = ChatCompletionRequestSystemMessageArgs::default()
        .content("你是一个助手名字叫 Heleny 。")
        .build()
        .expect("系统提示词失败");
    let user_msg = ChatCompletionRequestUserMessageArgs::default()
        .content("你是谁")
        .build()
        .expect("系统提示词失败");
    // Create request using builder pattern
    // Every request struct has companion builder struct with same name + Args suffix
    let request = CreateChatCompletionRequestArgs::default()
        .model("qwen-max") // 建议确认模型名称是否准确
        .messages([
            system_msg.into(), // 系统提示词通常排在第一条
            user_msg.into(),
        ])
        .n(1)
        .stream(true)
        .build()
        .expect("构造请求失败");
    // Call API
    let mut stream = client
        .chat() // Get the API "group" (responses, images, etc.) from the client
        .create_stream(request) // Make the API call in that "group"
        .await
        .expect("获取回复失败");
    while let Some(response) = stream.next().await {
        match response {
            Ok(ccr) => ccr.choices.iter().for_each(|c| {
                let chunk = match c.delta.content.clone() {
                    Some(chunk) => chunk,
                    None => return,
                };
                print!("{}", chunk);
            }),
            Err(e) => eprintln!("{}", e),
        }
    }
}

static SCHEMA: &'static str = r#"{
  "type": "object",
  "properties": {
    "content": {
      "type": "string",
      "description": "包含动作描述和回复文本的字符串，格式为：（动作）回复内容"
    },
    "need_help": {
      "oneOf": [
        { "type": "null" },
        { 
          "type": "string", 
          "description": "对用户需求的总结。如果需要外部组件（如 Planner 或 Executor）协助，则提供总结；否则为 null。" 
        }
      ]
    }
  },
  "required": ["content", "need_help"],
  "additionalProperties": false
}"#;

#[tokio::test]
async fn test_schema() {
    // Create client
    dotenvy::dotenv().ok();
    let api_key = std::env::var("GROK_API_KEY").expect("无 API KEY");
    let config = OpenAIConfig::new()
        .with_api_base("https://api.x.ai/v1")
        .with_api_key(api_key);
    let client = Client::with_config(config);
    let system_msg = ChatCompletionRequestSystemMessageArgs::default()
        .content("你是一个助手名字叫 Heleny 。")
        .build()
        .expect("系统提示词失败");
    let user_msg = ChatCompletionRequestUserMessageArgs::default()
        .content("你好呀")
        .build()
        .expect("系统提示词失败");
    // Create request using builder pattern
    // Every request struct has companion builder struct with same name + Args suffix
    let request = CreateChatCompletionRequestArgs::default()
        .model("grok-4-1-fast-non-reasoning") // 建议确认模型名称是否准确
        .messages([
            system_msg.into(), // 系统提示词通常排在第一条
            user_msg.into(),
        ])
        .n(1)
        .stream(true)
        .response_format(ResponseFormat::JsonSchema {
            json_schema: ResponseFormatJsonSchema {
                schema: Some(serde_json::from_str(SCHEMA).expect("解析成 Value 失败")),
                description: None,
                name: "math_reasoning".into(),
                strict: Some(true),
            },
        })
        .build()
        .expect("构造请求失败");
    // Call API
    let mut stream: ChatCompletionResponseStream = client
        .chat() // Get the API "group" (responses, images, etc.) from the client
        .create_stream(request) // Make the API call in that "group"
        .await
        .expect("获取回复失败");
    while let Some(response) = stream.next().await {
        match response {
            Ok(ccr) => ccr.choices.iter().for_each(|c| {
                let chunk = match c.delta.content.clone() {
                    Some(chunk) => chunk,
                    None => return,
                };
                print!("{}", chunk);
            }),
            Err(e) => eprintln!("{}", e),
        }
    }
}

#[tokio::test]
async fn test_gemini_api() {
    dotenvy::dotenv().ok();
    let api_key = std::env::var("GEMINI_API_KEY").expect("无 API KEY");

    let config = OpenAIConfig::new()
        .with_api_base("https://generativelanguage.googleapis.com/v1beta/openai/")
        .with_api_key(api_key);

    let client = Client::with_config(config);

    let user_msg = ChatCompletionRequestUserMessageArgs::default()
        .content("你是谁")
        .build()
        .unwrap();

    let request = CreateChatCompletionRequestArgs::default()
        .model("gemini-2.5-flash")
        .messages([ChatCompletionRequestMessage::User(user_msg)])
        .build()
        .unwrap();

    let resp = client.chat().create(request).await;

    match resp {
        Ok(r) => {
            let text = r
                .choices
                .get(0)
                .and_then(|c| c.message.content.clone())
                .unwrap_or_else(|| "<no content>".to_string());
            println!("OK: {}", text);
        }
        Err(e) => {
            eprintln!("ERR: {:#?}", e);
            panic!("获取回复失败");
        }
    }
}


use genai::chat::{ChatMessage, ChatRequest};
use genai::Client as GenaiClient;

const MODEL_GEMINI: &str = "gemini-2.5-flash";
const MODEL_CORE: &str = "core_24b_v.1-i1";

const MODEL_AND_KEY_ENV_NAME_LIST: &[(&str, &str)] = &[
	(MODEL_GEMINI, "GEMINI_API_KEY"),
    (MODEL_CORE, "")
];

#[tokio::test]
async fn test_genai_gemini() -> Result<(), Box<dyn std::error::Error>> {
	let question = "你是谁?";

	let chat_req = ChatRequest::new(vec![
		// -- Messages (de/activate to see the differences)
		ChatMessage::system("在一个句子回答"),
		ChatMessage::user(question),
	]);

	let client = GenaiClient::default();

	for (model, env_name) in MODEL_AND_KEY_ENV_NAME_LIST {
		// Skip if the environment name is not set
		if !env_name.is_empty() && std::env::var(env_name).is_err() {
			println!("===== Skipping model: {model} (env var not set: {env_name})");
			continue;
		}

		let adapter_kind = client.resolve_service_target(model).await?.model.adapter_kind;

		println!("\n===== MODEL: {model} ({adapter_kind}) =====");

		println!("\n--- Question:\n{question}");

		println!("\n--- Answer:");
        let option=ChatOptions::default();
		let chat_res = client.exec_chat(model, chat_req, Some(&option)).await?;
        
		println!("{}", chat_res.first_text().unwrap_or("NO ANSWER"));
        break;
	}

	Ok(())
}

#[tokio::test]
async fn test_gemini() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment variable
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable not set");

    // Create a Gemini client with default settings (Gemini 2.5 Flash)
    let client = Gemini::new(api_key)?;

    println!("basic content generation example starting");

    // Example 3: Multiple messages (conversation)
    let conversation_response = client
        .generate_content()
        .with_user_message("I'm learning to code.")
        .with_model_message("That's great! What programming language are you interested in?")
        .with_user_message("I want to learn Rust. Where should I start?")
        .execute()
        .await?;

    println!(
        "conversation response received {}",conversation_response.text()
    );

    println!("\n✅ Basic content generation examples completed successfully!");
    Ok(())
}


use async_openai::Embeddings;
#[tokio::test]
async fn test_embedding()->Result<()> {
    // Create client
    dotenvy::dotenv().ok();
    let config = OpenAIConfig::new()
        .with_api_base("http://127.0.0.1:1234/v1");
    let client = Client::with_config(config);

    let mut request=CreateEmbeddingRequest::default();
    request.model="text-embedding-bge-m3".into();
    request.input=EmbeddingInput::String("赫蕾妮".into());
    let embedding=Embeddings::new(&client).create(request).await?;

    println!("{:?}, len = {}", embedding, embedding.data.first().unwrap().embedding.len());
    Ok(())
}