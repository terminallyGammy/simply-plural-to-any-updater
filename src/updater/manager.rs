use crate::config::UserConfigForUpdater;
use crate::model::UserId;
use crate::updater::work_loop;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type SharedMutable<T> = Arc<Mutex<T>>;
pub type ThreadSafePerUser<T> = SharedMutable<HashMap<UserId, T>>;

#[derive(Clone)]
pub struct UpdaterManager {
    pub tasks: ThreadSafePerUser<work_loop::CancleableUpdater>,
    pub statuses: ThreadSafePerUser<work_loop::UserUpdatersStatuses>,
}

impl UpdaterManager {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            statuses: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_updaters_state(&self, user_id: &UserId) -> Result<work_loop::UserUpdatersStatuses> {
        Ok(self
            .statuses
            .lock()
            .map_err(|e| anyhow!(e.to_string()))?
            .get(user_id)
            .ok_or_else(|| anyhow!("No updaters found!"))?
            .to_owned())
    }

    pub fn set_updater_state(
        &self,
        user_id: &UserId,
        updater_state: work_loop::UserUpdatersStatuses,
    ) -> Result<()> {
        self.statuses
            .lock()
            .map_err(|e| anyhow!(e.to_string()))?
            .insert(user_id.to_owned(), updater_state);

        Ok(())
    }

    #[allow(clippy::significant_drop_tightening)]
    pub fn restart_updater(&self, user_id: &UserId, config: UserConfigForUpdater) -> Result<()> {
        let mut locked_task = self.tasks.lock().map_err(|e| anyhow!(e.to_string()))?;

        eprintln!("Aborting updater {user_id}");
        let _ = locked_task.get(user_id).map(tokio::task::JoinHandle::abort);

        let owned_self = self.to_owned();
        let new_task = tokio::spawn(async move {
            work_loop::run_loop(config, owned_self).await;
        });

        locked_task.insert(user_id.clone(), new_task);
        eprintln!("Restarted updater {user_id}");

        Ok(())
    }
}
