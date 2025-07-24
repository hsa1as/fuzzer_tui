use std::sync::{Arc, RwLock};

pub struct FuzzingWindow {
    pub context: Arc<RwLock<FuzzingWindowCtx>>,
}

pub struct FuzzingWindowCtx {}
