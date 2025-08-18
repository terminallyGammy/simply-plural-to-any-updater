use crate::config::UserConfigForUpdater;
use crate::model::UserId;
use crate::updater_loop;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type SharedMutable<T> = Arc<Mutex<T>>;
pub type ThreadSafePerUser<T> = SharedMutable<HashMap<UserId, T>>;

#[derive(Clone)]
pub struct SharedUpdaters {
    pub tasks: ThreadSafePerUser<updater_loop::CancleableUpdater>,
    pub statuses: ThreadSafePerUser<updater_loop::UserUpdatersStatuses>,
}

impl SharedUpdaters {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            statuses: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_updaters_state(
        &self,
        user_id: &UserId,
    ) -> Result<updater_loop::UserUpdatersStatuses> {
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
        updater_state: updater_loop::UserUpdatersStatuses,
    ) -> Result<()> {
        let mut locked_updater_states = self.statuses.lock().map_err(|e| anyhow!(e.to_string()))?;

        locked_updater_states.insert(user_id.to_owned(), updater_state);

        Ok(())
    }

    // todo. does this mean, that we block the global hashmap as long as this abort is happening???
    pub fn restart_updater(&self, user_id: &UserId, config: UserConfigForUpdater) -> Result<()> {
        let mut locked_task = self.tasks.lock().map_err(|e| anyhow!(e.to_string()))?;

        eprintln!("Aborting updater {user_id}");
        let _ = locked_task.get(user_id).map(tokio::task::JoinHandle::abort);

        let owned_self = self.clone();
        let new_task = tokio::spawn(async move {
            updater_loop::run_loop(config, owned_self).await;
        });

        locked_task.insert(user_id.clone(), new_task);
        eprintln!("Restarted updater {user_id}");

        Ok(())
    }
}
