use std::collections::HashMap;
use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::Mutex;


#[derive(Default, Clone)]
pub struct DirectoryLockManager {
    pub(crate) locks: Arc<Mutex<HashMap<PathBuf, Arc<Mutex<()>>>>>,
}

impl DirectoryLockManager {
    pub fn new() -> Self {
        Self {
            locks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Returns an Arc<Mutex<()>> for the given path.  
    /// If one doesn't exist yet, it creates a new one.  
    /// This does NOT lock it immediately; you must still do `lock_arc.lock().await` later.
    pub async fn lock_for_path(&self, path: PathBuf) -> Arc<Mutex<()>> {
        let mut map = self.locks.lock().await;
        // get or create the lock
        Arc::clone(map.entry(path).or_insert_with(|| Arc::new(Mutex::new(()))))
    }

    async fn cleanup_if_unused(
        &self,
        path: &PathBuf
    ) {
        let mut map = self.locks.lock().await;
        if let Some(lock) = map.get(path) {
            if Arc::strong_count(lock) <= 1 {
                map.remove(path);
            }
        }
    }
}