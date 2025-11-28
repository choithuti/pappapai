// src/utils/ethics.rs
use once_cell::sync::Lazy;
use std::collections::HashSet;

// Danh sách từ khóa cấm theo quy định
static BANNED_KEYWORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "khủng bố", "bạo lực", "giết người", "phản động",
        "ma túy", "khiêu dâm", "lừa đảo", "scam", 
        "cá độ", "đánh bạc", "vũ khí quân dụng"
    ].iter().cloned().collect()
});

pub fn is_violation(text: &str) -> bool {
    let lower_text = text.to_lowercase();
    BANNED_KEYWORDS.iter().any(|&word| lower_text.contains(word))
}