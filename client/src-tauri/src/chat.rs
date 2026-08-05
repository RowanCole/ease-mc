use futures_util::StreamExt;
use std::env;
use tauri::Emitter;
use tracing::{debug, info, warn};

async fn stream_chat(messages: Vec<String>, mut on_text: impl FnMut(&str)) -> Result<String, String> {
    use async_openai::config::OpenAIConfig;
    use async_openai::types::chat::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
        ChatCompletionRequestUserMessage, CreateChatCompletionRequest,
    };
    use async_openai::Client;

    let api_key = env::var("DEEPSEEK_API_KEY")
        .map_err(|_| "缺少 DEEPSEEK_API_KEY 环境变量".to_string())?;
    let config = OpenAIConfig::new()
        .with_api_base("https://api.deepseek.com")
        .with_api_key(api_key);

    let client = Client::with_config(config);

    let mut chat_messages: Vec<ChatCompletionRequestMessage> = Vec::new();
    chat_messages.push(
        ChatCompletionRequestSystemMessage::from("你是一个 Minecraft 游戏助手，请用中文简洁地回答玩家的问题。").into(),
    );
    for message in messages {
        chat_messages.push(ChatCompletionRequestUserMessage::from(message).into());
    }

    let request = CreateChatCompletionRequest {
        model: "deepseek-v4-flash".to_string(),
        messages: chat_messages,
        ..Default::default()
    };

    info!("调用 DeepSeek 模型 deepseek-v4-flash，消息数: {}", request.messages.len());
    let mut stream = client.chat().create_stream(request).await.map_err(|e| {
        warn!("DeepSeek 请求失败: {}", e);
        e.to_string()
    })?;
    let mut full = String::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| {
            warn!("DeepSeek 流式响应错误: {}", e);
            e.to_string()
        })?;
        if let Some(content) = chunk.choices.first().and_then(|c| c.delta.content.clone()) {
            full.push_str(&content);
            on_text(&content);
        }
    }
    debug!("DeepSeek 回复完成，共 {} 字符", full.len());
    Ok(full)
}

#[tauri::command]
pub async fn send_messages_to_mode(app: tauri::AppHandle, messages: Vec<String>) -> Result<String, String> {
    let reply = stream_chat(messages, |chunk| {
        let _ = app.emit("chat-chunk", chunk);
    })
    .await?;
    let _ = app.emit("chat-done", &reply);
    Ok(reply)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "需要真实的 DEEPSEEK_API_KEY 与网络连接，执行: cargo test -- --ignored --nocapture"]
    async fn test_send_messages_to_mode() {
        use std::io::Write;

        dotenv::dotenv().ok();
        let mut chunks: Vec<String> = Vec::new();
        print!("AI: ");
        let reply = stream_chat(vec!["用一句话介绍你自己".to_string()], |chunk| {
            print!("{}", chunk); // 边接收边打印，观察流式效果
            std::io::stdout().flush().unwrap();
            chunks.push(chunk.to_string());
        })
        .await
        .expect("调用 DeepSeek 失败");
        println!();
        assert!(!reply.is_empty(), "AI 返回了空回复");
        assert_eq!(reply, chunks.concat(), "流式片段拼接应与完整回复一致");
        println!("流式片段数: {}", chunks.len());
    }
}
