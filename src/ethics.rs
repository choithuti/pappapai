// src/ethics.rs
pub struct EthicsFilter;

impl EthicsFilter {
    // Danh sách từ khóa cấm theo quy định (Demo rút gọn)
    const BLACKLIST: [&'static str; 10] = [
        "phản động", "khủng bố", "lật đổ", "bạo loạn", 
        "ma túy", "đánh bạc", "cá độ", "vũ khí",
        "khiêu dâm", "lừa đảo"
    ];

    pub fn check(content: &str) -> Result<(), String> {
        let lower_content = content.to_lowercase();
        
        for &word in Self::BLACKLIST.iter() {
            if lower_content.contains(word) {
                return Err(format!(
                    "⚠️ NỘI DUNG BỊ TỪ CHỐI: Vi phạm quy tắc an toàn thông tin & Luật An ninh mạng (Phát hiện từ khóa: '{}').", 
                    word
                ));
            }
        }
        Ok(())
    }
}