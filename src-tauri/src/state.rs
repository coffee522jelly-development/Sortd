use std::sync::Mutex;
use crate::models::MoveOp;

/// 移動操作の履歴をバッチ単位で管理
pub struct History {
    pub batches: Vec<Vec<MoveOp>>,
}

/// アプリケーションの共有状態（履歴を保持）
pub struct AppState {
    pub history: Mutex<History>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(History { batches: Vec::new() }),
        }
    }
}
