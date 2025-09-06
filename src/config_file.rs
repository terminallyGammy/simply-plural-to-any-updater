use anyhow::Result;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

/// Name of the default configuration file created next to the executable.
const DEFAULT_CONFIG_NAME: &str = "sp2any.json";

/// Ensure that the configuration file exists.
///
/// If the file is missing, it will be created with placeholder content so
/// that users can fill in the required credentials.  The path to the
/// configuration file is returned for informational purposes.
pub fn ensure_config_file() -> Result<PathBuf> {
    let path = PathBuf::from(DEFAULT_CONFIG_NAME);

    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = File::create(&path)?;
        let default_content = r#"{
  "simply_plural_token": "",
  "discord_status_message_token": "",
  "vrchat_username": "",
  "vrchat_password": ""
}
"#;
        file.write_all(default_content.as_bytes())?;
    }

    Ok(path)
}
