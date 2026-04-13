use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tauri::{AppHandle, Emitter};
use crate::lock_session::{LockSessionManager, WatcherState};

pub struct WatcherSupervisor {
    app_handle: AppHandle,
    session_manager: Arc<LockSessionManager>,
}

impl WatcherSupervisor {
    pub fn new(app_handle: AppHandle, session_manager: Arc<LockSessionManager>) -> Self {
        Self {
            app_handle,
            session_manager,
        }
    }

    pub async fn run(&self) {
        let mut restart_attempts = 0;
        let max_attempts = 5;

        loop {
            
            let state = *self.session_manager.watcher_state.read().unwrap();
            
            if state == WatcherState::Crashed {
                if restart_attempts < max_attempts {
                    restart_attempts += 1;
                    let backoff = Duration::from_secs(2u64.pow(restart_attempts as u32));
                    
                    println!("Watcher crashed. Restart attempt {}/{} in {:?}", restart_attempts, max_attempts, backoff);
                    
                    self.app_handle.emit("watcher_restarted", serde_json::json!({
                        "attempt_number": restart_attempts
                    })).unwrap();

                    sleep(backoff).await;
                    let mut state_write = self.session_manager.watcher_state.write().unwrap();
                    *state_write = WatcherState::Running;
                } else {
                    println!("Watcher failed permanently after {} attempts", max_attempts);
                    self.app_handle.emit("watcher_failed_permanently", serde_json::json!({
                        "reason": "Max restart attempts reached"
                    })).unwrap();
                    break;
                }
            }

            sleep(Duration::from_secs(2)).await;
        }
    }
}
