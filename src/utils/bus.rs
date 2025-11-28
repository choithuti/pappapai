// src/utils/bus.rs
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct GlobalBus {
    // Channel gửi tuple: (Topic/Target, Data)
    tx: broadcast::Sender<(String, Vec<u8>)>,
}

impl GlobalBus {
    pub fn new() -> Self {
        // Capacity 4096 messages để tránh bị lag
        let (tx, _) = broadcast::channel(4096);
        Self { tx }
    }

    // Gửi tin nhắn
    pub fn publish(&self, target: &str, data: Vec<u8>) {
        // Bỏ qua lỗi nếu không có ai nghe (send error)
        let _ = self.tx.send((target.to_string(), data));
    }

    // Đăng ký nhận tin nhắn
    pub fn subscribe(&self) -> (broadcast::Receiver<(String, Vec<u8>)>, broadcast::Sender<(String, Vec<u8>)>) {
        (self.tx.subscribe(), self.tx.clone())
    }
}