// 下载进度上报：基于原子计数按字节推进，超过阈值节流发送，避免高频 emit。
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use tauri::Emitter;

#[derive(Clone)]
pub struct ProgressCtx {
    app: tauri::AppHandle,
    done: Arc<AtomicU64>,
    last_percent: Arc<AtomicU64>,
    start: f64,
    span: f64,
    total_bytes: u64,
}

impl ProgressCtx {
    pub fn new(app: tauri::AppHandle, start: f64, end: f64, total_bytes: u64) -> Self {
        Self {
            app,
            done: Arc::new(AtomicU64::new(0)),
            last_percent: Arc::new(AtomicU64::new(0)),
            start,
            span: end - start,
            total_bytes,
        }
    }

    pub fn add_bytes(&self, n: u64, response_len: u64) {
        let total = if self.total_bytes > 0 {
            self.total_bytes
        } else {
            response_len
        };
        if total == 0 {
            return;
        }
        let bytes = self.done.fetch_add(n, Ordering::SeqCst) + n;
        let percent = (self.start + self.span * (bytes as f64 / total as f64))
            .min(self.start + self.span);
        let key = (percent * 1000.0) as u64;
        if key >= self.last_percent.load(Ordering::SeqCst) + 10 {
            self.last_percent.store(key, Ordering::SeqCst);
            let _ = self
                .app
                .emit("download-progress", serde_json::json!({ "percent": percent }));
        }
    }

    /// 将进度收敛到区间终点（下载成功或 Content-Length 缺失时的兜底）
    pub fn finish(&self) {
        let end = self.start + self.span;
        let _ = self
            .app
            .emit("download-progress", serde_json::json!({ "percent": end }));
    }
}
