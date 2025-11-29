use reqwest::Client;
use serde_json::json;
use std::error::Error;

#[derive(Clone)]
pub struct LLMBridge {
    client: Client,
    api_url: String,
    api_key: String,
    model: String,
}

impl LLMBridge {
    pub fn new() -> Self {
        // Cấu hình mặc định (Bạn có thể đổi sang OpenAI, OpenRouter, hoặc Local Ollama)
        // Ví dụ dùng OpenRouter (Miễn phí một số model) hoặc DeepSeek
        Self {
            client: Client::new(),
            // Dùng OpenRouter để truy cập nhiều model (hoặc thay bằng https://api.openai.com/v1/chat/completions)
            api_url: "https://openrouter.ai/api/v1/chat/completions".to_string(), 
            // Lấy key từ biến môi trường hoặc điền trực tiếp (KHÔNG KHUYẾN KHÍCH điền trực tiếp nếu public code)
            api_key: std::env::var("LLM_API_KEY").unwrap_or("".to_string()),
            model: "google/gemini-2.0-flash-lite-preview-02-05:free".to_string(), // Model miễn phí trên OpenRouter
        }
    }

    pub async fn ask_ai(&self, prompt: &str) -> Result<String, Box<dyn Error>> {
        if self.api_key.is_empty() {
            return Err("Chưa cấu hình API Key cho LLM!".into());
        }

        println!("🤖 ASKING SUPER-AI (Model: {}): '{}'...", self.model, prompt);

        let body = json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": "Bạn là một siêu trí tuệ hỗ trợ cho Pappap AI Node. Hãy trả lời ngắn gọn, súc tích và chính xác."},
                {"role": "user", "content": prompt}
            ]
        });

        let resp = self.client.post(&self.api_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if resp.status().is_success() {
            let json: serde_json::Value = resp.json().await?;
            // Lấy nội dung trả lời từ JSON chuẩn OpenAI format
            if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
                return Ok(content.trim().to_string());
            }
        } else {
            let error_text = resp.text().await?;
            println!("❌ LLM Error: {}", error_text);
        }

        Err("Không nhận được câu trả lời từ AI.".into())
    }
}
