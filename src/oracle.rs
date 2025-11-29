use reqwest::Client;
use scraper::{Html, Selector};
use serde_json::Value;
use std::error::Error;
use regex::Regex;

pub struct Oracle {
    client: Client,
}

impl Oracle {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0.0.0 Safari/537.36")
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap(),
        }
    }

    pub async fn smart_search(&self, query: &str) -> Result<String, Box<dyn Error>> {
        let q_lower = query.to_lowercase();

        // 1. TÀI CHÍNH
        if q_lower.contains("bitcoin") || q_lower.contains("eth") || q_lower.contains("coin") {
            return self.get_crypto_price(&q_lower).await;
        }
        if q_lower.contains("vàng") || q_lower.contains("sjc") {
            // Tìm thẳng tin tức mới nhất về giá vàng
            return self.search_duckduckgo(&format!("Giá vàng SJC hôm nay {} mới nhất", chrono::Local::now().format("%d/%m"))).await;
        }

        // 2. CHÍNH TRỊ / THỜI SỰ (Ưu tiên DuckDuckGo tin tức hơn là Wiki)
        if q_lower.contains("tổng bí thư") || q_lower.contains("chủ tịch") || q_lower.contains("hiện tại") {
             // Thử tìm DuckDuckGo trước để lấy tin mới nhất
             if let Ok(res) = self.search_duckduckgo(query).await {
                 if !res.contains("Không tìm thấy") { return Ok(res); }
             }
        }

        // 3. ĐỊNH NGHĨA (Wikipedia)
        if let Ok(wiki) = self.search_wikipedia(query).await {
            return Ok(wiki);
        }

        // 4. CUỐI CÙNG
        self.search_duckduckgo(query).await
    }

    async fn get_crypto_price(&self, query: &str) -> Result<String, Box<dyn Error>> {
        let coin = if query.contains("eth") { "ethereum" } else { "bitcoin" };
        let url = format!("https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd,vnd", coin);
        let resp = self.client.get(&url).send().await?.json::<Value>().await?;
        let usd = resp[coin]["usd"].as_f64().unwrap_or(0.0);
        let vnd = resp[coin]["vnd"].as_f64().unwrap_or(0.0);
        Ok(format!("1 {} = ${} (~ {} VNĐ)", coin.to_uppercase(), usd, vnd))
    }

    async fn search_wikipedia(&self, query: &str) -> Result<String, Box<dyn Error>> {
        let url = format!("https://vi.wikipedia.org/w/api.php?action=query&list=search&srsearch={}&format=json&srlimit=1", query);
        let resp = self.client.get(&url).send().await?.json::<Value>().await?;
        let page_id = resp["query"]["search"][0]["pageid"].as_u64().unwrap_or(0);
        if page_id > 0 {
            let c_url = format!("https://vi.wikipedia.org/w/api.php?action=query&prop=extracts&exintro&explaintext&pageids={}&format=json", page_id);
            let c_resp = self.client.get(&c_url).send().await?.json::<Value>().await?;
            let text = c_resp["query"]["pages"][page_id.to_string()]["extract"].as_str().unwrap_or("");
            return Ok(text.chars().take(400).collect::<String>() + "...");
        }
        Err("Wiki not found".into())
    }

    async fn search_duckduckgo(&self, query: &str) -> Result<String, Box<dyn Error>> {
        let url = format!("https://html.duckduckgo.com/html/?q={}", query);
        let resp = self.client.get(&url).send().await?.text().await?;
        let doc = Html::parse_document(&resp);
        let sel = Selector::parse(".result__snippet").unwrap();
        
        if let Some(el) = doc.select(&sel).next() {
            let text = el.text().collect::<Vec<_>>().join("");
            let re = Regex::new(r"\s+").unwrap();
            return Ok(re.replace_all(&text, " ").trim().to_string());
        }
        Ok("Không tìm thấy thông tin.".to_string())
    }
}
