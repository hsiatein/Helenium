use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::chat::ChatCompletionRequestMessage;
use async_openai::types::chat::ChatCompletionRequestSystemMessageArgs;
use async_openai::types::chat::ChatCompletionRequestUserMessageArgs;
use async_openai::types::chat::ChatCompletionResponseStream;
use async_openai::types::chat::CreateChatCompletionRequestArgs;
use async_openai::types::chat::ResponseFormat;
use async_openai::types::chat::ResponseFormatJsonSchema;
use async_openai::types::responses::CreateResponseArgs;
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
