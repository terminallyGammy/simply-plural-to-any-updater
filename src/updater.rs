use anyhow::Result;
use serde::Serialize;

use crate::{
    config::Config, discord::DiscordUpdater, simply_plural::Fronter, vrchat::VRChatUpdater,
};

#[derive(Clone, Serialize, strum_macros::Display)]
pub enum Platform {
    VRChat,
    Discord,
}

#[derive(Clone, Serialize, strum_macros::Display)]
pub enum UpdaterStatus {
    Inactive,
    // Paused,
    Running,
    // Error,
    // Unknown,
}

pub enum Updater {
    VRChat(Box<VRChatUpdater>),
    Discord(DiscordUpdater),
}

impl Updater {
    pub fn new(platform: Platform) -> Self {
        match platform {
            Platform::VRChat => Self::VRChat(Box::new(VRChatUpdater::new(platform))),
            Platform::Discord => Self::Discord(DiscordUpdater::new(platform)),
        }
    }

    pub const fn platform(&self) -> Platform {
        match self {
            Self::VRChat(_) => Platform::VRChat,
            Self::Discord(_) => Platform::Discord,
        }
    }

    pub const fn enabled(&self, config: &Config) -> bool {
        match self {
            Self::VRChat(_) => config.enable_vrchat,
            Self::Discord(_) => config.enable_discord,
        }
    }

    pub async fn setup(&mut self, config: &Config) -> Result<()> {
        match self {
            Self::VRChat(updater) => updater.setup(config).await,
            Self::Discord(updater) => updater.setup(config).await,
        }
    }

    pub async fn update_fronting_status(
        &mut self,
        config: &Config,
        fronts: &[Fronter],
    ) -> Result<()> {
        match self {
            Self::VRChat(updater) => updater.update_fronting_status(config, fronts).await,
            Self::Discord(updater) => updater.update_fronting_status(config, fronts).await,
        }
    }
}
