use std::sync::Arc;

use tokio::sync::Mutex;

pub type BMArc<T> = Arc<Mutex<Box<T>>>;
pub type MArc<T> = Arc<Mutex<T>>;
