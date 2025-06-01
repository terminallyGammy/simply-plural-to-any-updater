use crate::{config::{self, Config}, simply_plural};
use encoding_rs::{self, ISO_8859_15};
use anyhow::{anyhow, Result};
use reqwest::{cookie::{self, CookieStore}, Url};
use std::{
    io::{self, Write}, str::FromStr, sync::{self}, thread, time
};
use time::Duration;
use vrchatapi::{
    apis::{authentication_api, configuration::Configuration, users_api},
    models::{EitherUserOrTwoFactor, TwoFactorAuthCode, TwoFactorEmailCode, UpdateUserRequest},
};
use chrono;

const VRCHAT_UPDATER_USER_AGENT: &str = concat!(
    "SimplyPluralToVRChatUpdater/",
    env!("CARGO_PKG_VERSION"),
    " golly.ticker",
    "@",
    "gmail.com"
);

const VRCHAT_COOKIE_URL: &str = "https://api.vrchat.cloud";
const VRCHAT_MAX_ALLOWED_STATUS_LENGTH: usize = 23;

pub async fn run_updater_loop(config: &Config) -> Result<()> {
    let (vrchat_config, user_id) = authenticate_vrchat(config).await?;

    loop {
        eprintln!("\n\n======================= UTC {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"));

        let fronts = crate::simply_plural::fetch_fronts(&config).await?;
        
        update_fronts_in_vrchat_status(&config, &vrchat_config, &user_id, fronts).await?;
        
        eprintln!("Waiting {}s for next update trigger...", config.wait_seconds);
        thread::sleep(Duration::from_secs(config.wait_seconds));
    }
}

async fn update_fronts_in_vrchat_status(
    config: &Config,
    vrchat_config: &Configuration,
    user_id: &String,
    fronts: Vec<simply_plural::MemberContent>,
) -> Result<()> {
    let status_string = format_vrchat_status(config, fronts);

    let mut update_request = UpdateUserRequest::new();
    update_request.status_description = Some(status_string.clone());

    match users_api::update_user(vrchat_config, &user_id, Some(update_request)).await {
        Ok(_) => eprintln!("VRChat status updated successfully to: '{}'", status_string),
        Err(err) => eprintln!("VRChat status failed to be updated. Error: {}", err),
    }

    Ok(())
}

fn format_vrchat_status(config: &Config, fronts: Vec<simply_plural::MemberContent>) -> String {
    let cleaned_fronter_names: Vec<String> = if fronts.is_empty() {
        vec![config.vrchat_updater_no_fronts.clone()] // Use configured string if no fronters
    } else {
        fronts.iter().map(|m| {
            let name = if m.info.vrchat_status_name.is_empty() { &m.name } else { &m.info.vrchat_status_name };
            clean_name_for_vrchat(name)
        }).collect()
    };
    eprintln!("Cleaned fronter names for status: {:?}", cleaned_fronter_names);

    // Convert Vec<String> to Vec<&str> for convenient joining and slicing.
    let fronter_names_as_str: Vec<&str> = cleaned_fronter_names.iter().map(String::as_str).collect();

    let long_string = format!("{} {}", config.vrchat_updater_prefix.as_str(), fronter_names_as_str.join(", "));
    let short_string = format!("{}{}", config.vrchat_updater_prefix.as_str(), fronter_names_as_str.join(","));
    let truncated_string = {
        let truncated_names: Vec<String> = fronter_names_as_str
            .iter()
            .map(|&name| {
                let mut truncated_name = String::new();
                
                let _ = &name.chars()
                    .take(config.vrchat_updater_truncate_names_to)
                    .for_each(|c| truncated_name.push(c));
                
                truncated_name
            })
            .collect();
        format!("{}{}", config.vrchat_updater_prefix.as_str(), truncated_names.join(",").as_str())
    };

    eprintln!("Long      string: '{}' ({})", long_string, long_string.len());
    eprintln!("Short     string: '{}' ({})", short_string, short_string.len());
    eprintln!("Truncated string: '{}' ({})", truncated_string, truncated_string.len());
    
    if long_string.len() <= VRCHAT_MAX_ALLOWED_STATUS_LENGTH { long_string }
    else if short_string.len() <= VRCHAT_MAX_ALLOWED_STATUS_LENGTH { short_string }
    else { truncated_string }
}


// VRChat status messages does not display all UTF-8 characters.
// This function removes all characters which are not of a specific encoding from the string.
// We also trim the name, in case the cleanup made new spaces appear.
fn clean_name_for_vrchat(dirty_name: &str) -> String {
    let mut iso_filtered_name = String::new();
    
    for ch in dirty_name.chars() {
        // Convert char utf-8 str
        let ch_string = ch.to_string();

        // convert utf-8 str to the limited encoding and check if the character is supported.
        let mut char_cleaned_buffer = [0u8; 20];
        let (_, _, _, is_unsupported_character) = ISO_8859_15
            .new_encoder()
            .encode_from_utf8(&ch_string.as_str(), &mut char_cleaned_buffer, true);

        if !is_unsupported_character {
            iso_filtered_name.push(ch);
        }
    }

    // remove consecutive whitespace resulting from cleanup. also trims string.
    iso_filtered_name
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

async fn authenticate_vrchat(config: &Config) -> Result<(Configuration, String)> {
    let mut vrchat_config = Configuration::default();
    vrchat_config.user_agent = Some(VRCHAT_UPDATER_USER_AGENT.to_string());
    vrchat_config.basic_auth = Some((config.vrchat_username.clone(), Some(config.vrchat_password.clone())));

    let cookie_store = sync::Arc::new(cookie::Jar::default());
    let cookie_url = &Url::from_str(VRCHAT_COOKIE_URL).unwrap();

    let mut cookie_exists = !config.vrchat_cookie.is_empty();

    if cookie_exists {
        cookie_store.add_cookie_str(&config.vrchat_cookie, cookie_url);
    };

    vrchat_config.client = reqwest::Client::builder()
        .cookie_provider(cookie_store.clone())
        .build()
        .unwrap();
    
    match authentication_api::get_current_user(&vrchat_config).await.unwrap() {
        EitherUserOrTwoFactor::CurrentUser(_me) => true,
        EitherUserOrTwoFactor::RequiresTwoFactorAuth(requires_auth) => {
            // either cookie was empty or invalid. we mark the cookie as such then
            cookie_exists = false;
            if requires_auth.requires_two_factor_auth.contains(&String::from("emailOtp")) {
                let code = read_user_input(&format!("Your account {} has received an Email with a 2FA code. Please enter it: ", config.vrchat_username));
                authentication_api::verify2_fa_email_code(&vrchat_config, TwoFactorEmailCode::new(code)).await?.verified
            } else {
                let code = read_user_input(&format!("Please enter your Authenticator 2FA code for the account {}:", config.vrchat_username));
                authentication_api::verify2_fa(&vrchat_config, TwoFactorAuthCode::new(code)).await?.verified
            }
        }
    };

    if !cookie_exists {
        let cookie_value = cookie_store.cookies(cookie_url).unwrap();
        config::store_vrchat_cookie(cookie_value.to_str().unwrap()).await?;
    }

    match authentication_api::get_current_user(&vrchat_config).await.unwrap() {
        EitherUserOrTwoFactor::CurrentUser(user) => Ok((vrchat_config, user.id)),
        EitherUserOrTwoFactor::RequiresTwoFactorAuth(_) => Err(anyhow!("Cookie invalid for user {}", config.vrchat_username)),
    }
}

fn read_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().expect("Failed to flush stdout");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    input.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::simply_plural::{MemberContent, MemberContentInfo};

    fn mock_config_for_format_tests(prefix: &str, no_fronts: &str, name_truncate_to: usize) -> Config {
        let mut config = Config::default();
        config.vrchat_updater_prefix = prefix.to_string();
        config.vrchat_updater_no_fronts = no_fronts.to_string();
        config.vrchat_updater_truncate_names_to = name_truncate_to;
        config
    }

    // Helper function to create mock MemberContent
    fn mock_member_content(name: &str, vrchat_status_name: &str) -> MemberContent {
        MemberContent {
            name: name.to_string(),
            avatar_url: String::new(),
            info: MemberContentInfo {
                vrchat_status_name: vrchat_status_name.to_string(),
            },
        }
    }

    #[test]
    fn test_format_vrchat_status_empty_fronts() {
        let config = mock_config_for_format_tests("F:", "nobody?", 3);
        let fronts = vec![];
        assert_eq!(format_vrchat_status(&config, fronts), "F: nobody?");
    }

    #[test]
    fn test_format_vrchat_status_single_member_fits_long_string() {
        let config = mock_config_for_format_tests("F:", "N/A", 3);
        let fronts = vec![mock_member_content("Alice", "")]; // "P: Alice" (8 chars)
        assert_eq!(format_vrchat_status(&config, fronts), "F: Alice");
    }

    #[test]
    fn test_format_vrchat_status_multiple_members_fit_long_string() {
        let config = mock_config_for_format_tests("F:", "N/A", 3);
        let fronts = vec![
            mock_member_content("Alice", ""),
            mock_member_content("Bob", ""),
        ]; // "P: Alice, Bob" (13 chars)
        assert_eq!(format_vrchat_status(&config, fronts), "F: Alice, Bob");
    }

    #[test]
    fn test_format_vrchat_status_fits_short_string_not_long() {
        // VRCHAT_MAX_ALLOWED_STATUS_LENGTH is 23
        let config = mock_config_for_format_tests("Status:", "N/A", 3);
        let fronts = vec![
            mock_member_content("UserOne", ""),
            mock_member_content("UserTwo", ""),
        ];
        // Long: "Status: UserOne, UserTwo" (24 chars) > 23
        // Short: "Status:UserOne,UserTwo" (23 chars) <= 23
        assert_eq!(format_vrchat_status(&config, fronts), "Status:UserOne,UserTwo");
    }

    #[test]
    fn test_format_vrchat_status_fits_truncated_string_not_short() {
        let config = mock_config_for_format_tests("F:", "N/A", 3);
        let fronts = vec![
            mock_member_content("Alexander", ""),
            mock_member_content("Benjamin", ""),
            mock_member_content("Charlotte", ""),
        ];
        // Long: "P: Alexander, Benjamin, Charlotte" 33 > 23
        // Short: "P:Alexander,Benjamin,Charlotte" 31 > 23
        // Truncated: "P:Ale,Ben,Cha" 14 <= 23
        assert_eq!(format_vrchat_status(&config, fronts), "F:Ale,Ben,Cha");
    }

    #[test]
    fn test_format_vrchat_status_uses_vrchat_status_name() {
        let config = mock_config_for_format_tests("F:", "N/A", 3);
        let fronts = vec![mock_member_content("OriginalName", "VRChatSpecific")];
        assert_eq!(format_vrchat_status(&config, fronts), "F: VRChatSpecific");
    }

    #[test]
    fn test_format_vrchat_status_cleans_names() {
        let config = mock_config_for_format_tests("F:", "N/A", 3);
        let fronts = vec![mock_member_content("User😊Name", "")];
        assert_eq!(format_vrchat_status(&config, fronts), "F: UserName");
    }

    #[test]
    fn test_format_vrchat_status_complex_truncation_and_vrc_name() {
        let config = mock_config_for_format_tests("F:", "N/A", 4);
        let fronts = vec![
            mock_member_content("LongNameOne😊", ""),
            mock_member_content("Shorty", "VRC11"),
            mock_member_content("AnotherVeryLong", "")
        ];
        // Cleaned names for status: LongNameOne, VRC11, AnotherVeryLong
        // Long: "F: LongNameOne, VRC11, AnotherVeryLong" 38 > 23
        // Short: "F:LongNameOne,VRC11,AnotherVeryLong" 36 > 23
        // Truncated names: Long, VRC1, Anot
        // Truncated string: "F:Long,VRC1,Anot" 17 <= 23
        assert_eq!(format_vrchat_status(&config, fronts), "F:Long,VRC1,Anot");
    }

    #[test]
    fn test_clean_name_for_vrchat_encoding_and_whitespace() {
        assert_eq!(
            clean_name_for_vrchat("ValidName123!€ Špecial Chars Ž"),
            "ValidName123!€ Špecial Chars Ž",
            "Should keep all valid ISO_8859_15 characters"
        );

        assert_eq!(
            clean_name_for_vrchat("Name😊With🚀Emojis❤️Symbols✅"),
            "NameWithEmojisSymbols",
            "Should remove emojis"
        );

        assert_eq!(
            clean_name_for_vrchat("Héllo Wörld🎉"),
            "Héllo Wörld",
            "Should handle mixed valid and invalid characters"
        );

        assert_eq!(
            clean_name_for_vrchat("  Trimmed  From  Name  "),
            "Trimmed From Name",
            "Should collapse consecutive spaces and trim"
        );

        assert_eq!(
            clean_name_for_vrchat(""),
            ""
        );

        assert_eq!(
            clean_name_for_vrchat("😊🚀🎉"),
            ""
        );

        assert_eq!(
            clean_name_for_vrchat("   \t\n   "),
            ""
        );

        assert_eq!(
            clean_name_for_vrchat("你好WorldПриветUser1"),
            "WorldUser1",
            "Should remove characters from other scripts like Hanzi or Cyrillic"
        );

        assert_eq!(
            clean_name_for_vrchat("A 😊B C🚀D"),
            "A B CD",
            "Should collapse spaces created by invalid characters"
        );
    }
}
