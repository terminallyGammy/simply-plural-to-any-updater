#[macro_use] extern crate rocket;

use rocket::{response::{self, content::RawHtml}, State};
use serde::Deserialize;
use vrchatapi::{apis::authentication_api, models::{EitherUserOrTwoFactor, TwoFactorAuthCode, TwoFactorEmailCode, UpdateUserRequest}};
use vrchatapi::apis::configuration::Configuration;
use std::{env, fmt::Debug, io::{self, Write}, string::String, thread, time, vec};
use time::Duration;
use reqwest::Client;
use anyhow::{anyhow, Error, Result};
use tokio;
use chrono;

const VRCHAT_UPDATER_USER_AGENT: &str = concat!("SimplyPluralToVRChatUpdater/",env!("CARGO_PKG_VERSION")," golly.ticker","@","gmail.com");
// the email is written in a slightly obfuscated way for automatic scrapers to not find it.

const VRCHAT_MAX_ALLOWED_STATUS_LENGTH: usize = 23;

#[tokio::main]
async fn main() -> Result<()> {
    eprintln!("Starting VRChat SPS status updater...");

    let config = load_config().await?;

    if config.serve_api {
        start_server(&config).await?
    }
    else {
        update_vrchat_status_fronts_loop(&config).await?
    }

    Ok(())
}



async fn start_server(config: &Config) -> Result<()> {
    rocket::build()
        .manage(config.clone())
        .mount("/", routes![rest_get_fronting])
        .launch()
        .await
        .map_err(|e| anyhow::anyhow!("Rocket failed with: {}",e))
        .map(|_| ())
}



#[get("/fronting")]
async fn rest_get_fronting(config: &State<Config>) -> Result<RawHtml<String>, response::Debug<Error>> {
    let fronts = fetch_fronts(&config).await?;
    let html = generate_html(&config, fronts);

    Ok( RawHtml(html) )
}



async fn update_vrchat_status_fronts_loop(config: &Config) -> Result<()>{
    let (vrchat_config, user_id) = authenticate_vrchat(config).await?;

    loop {
        eprintln!("\n\n======================= UTC {}",chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"));

        let fronts = fetch_fronts(&config).await?;
        
        update_fronts_in_vrchat_status(&config, &vrchat_config, &user_id, fronts).await?;
        
        eprintln!("Waiting {}s for next update trigger...", config.wait_seconds);
        thread::sleep(Duration::from_secs(config.wait_seconds));
    }
}



async fn enrich_fronter_ids_with_member_info(front_entries: Vec<FrontEntry>, config: &Config) -> Result<Vec<MemberContent>> {
    let system_id = &front_entries[0].content.uid;
    let front_uids: Vec<String>  = front_entries.iter().map(|e| e.content.member.clone()).collect();

    let fronts_url = format!("{}/members/{}", &config.sps_base_url, system_id);
    eprintln!("Fetching all members from SPS: {}", fronts_url);
    let members_response = config
        .client
        .get(&fronts_url)
        .header("Authorization", &config.sps_token)
        .send()
        .await?;
    eprintln!("Status: {}", members_response.status());

    let members: Vec<Member> = members_response.error_for_status()?.json().await?;

    let fronting_members: Vec<MemberContent> = members
        .into_iter()
        .filter(|m| front_uids.contains(&m.id))
        .map(|m| {
            eprintln!("Fronting member: {:?}",m.content);
            m.content
        })
        .collect();

    return Ok(fronting_members);
}   




async fn fetch_fronts(config: &Config) -> Result<Vec<MemberContent>> {
    // 1. Fetch current fronts from Simply Plural
    let fronts_url = format!("{}/fronters", &config.sps_base_url);
    eprintln!("Fetching fronts from SPS: {}", fronts_url);
    let fronts_response = config
        .client
        .get(&fronts_url)
        .header("Authorization", &config.sps_token)
        .send()
        .await?;
    eprintln!("Status: {}", fronts_response.status());
    
    let front_entries: Vec<FrontEntry> = fronts_response
        .error_for_status()?
        .json()
        .await?;

    let fronter_ids: Vec<&String> = front_entries.iter().map(|e| &e.content.member).collect();
    eprintln!("Fronter IDs: {:?}", fronter_ids);

    let fronts: Vec<MemberContent> = if fronter_ids.is_empty() { vec![] } else { enrich_fronter_ids_with_member_info(front_entries, config).await? };

    Ok(fronts)
}


async fn update_fronts_in_vrchat_status(config: &Config, vrchat_config: &Configuration, user_id: &String, fronts: Vec<MemberContent>) -> Result<()> {
    
    // 1. Build elements of status string
    let status_string = format_vrchat_status(config, fronts);

    // 2. Build request
    let mut update_request = UpdateUserRequest::new();
    update_request.status_description = Some(status_string.clone());
    
    // 3. Send status update request to VRChat
    // Uses https://vrchatapi.github.io/docs/api/ PUT Update User Info
    match vrchatapi::apis::users_api::update_user(
        vrchat_config,
        &user_id,
        Some(update_request)
    ).await
    {
        Ok(_) => eprintln!("VRChat status updated successfully to: {}", status_string),
        Err(err) => eprintln!("VRChat status failed to be updated. Error: {}", err),
    }

    // always return OK. even if request failed. we don't want to abort
    Ok(())
}

fn format_vrchat_status(config: &Config, fronts: Vec<MemberContent>) -> String {
    let fronter_strings: Vec<&str> = if fronts.is_empty() {
        vec![&config.vrchat_updater_no_fronts]
    } else {
        // todo. only keep ascii chars.
        fronts.iter().map(|m|m.name.as_str()).collect()
    };
    eprintln!("Status string elements: {:?}", fronter_strings);

    let long_string = format!("{} {}", config.vrchat_updater_prefix.as_str(), fronter_strings.join(", "));
    let short_string = format!("{}{}", config.vrchat_updater_prefix.as_str(), fronter_strings.join(","));
    let truncated_string = {
        let prefix_slice = 0..config.vrchat_updater_truncate_names_to;
        let truncated_names: Vec<&str> = fronter_strings
            .iter()
            .map(|name|name.get(prefix_slice.clone())
            .unwrap_or_default())
            .collect();
        format!("{}{}", config.vrchat_updater_prefix.as_str(), truncated_names.join(","))
    };

    eprintln!("Long      string: '{}' ({})", long_string, long_string.len());
    eprintln!("Short     string: '{}' ({})", short_string, short_string.len());
    eprintln!("Truncated string: '{}' ({})", truncated_string, truncated_string.len());

    // use long string, if possible
    if long_string.len() <= VRCHAT_MAX_ALLOWED_STATUS_LENGTH { long_string }
    // otherwise try small string
    else if short_string.len() <= VRCHAT_MAX_ALLOWED_STATUS_LENGTH { short_string }
    // if that's also too long, then truncate the names
    else { truncated_string }
}



// Find examples in https://github.com/vrchatapi/vrchatapi-rust
async fn authenticate_vrchat(config: &Config) -> Result<(Configuration,String)> {
    let mut vrchat_config = vrchatapi::apis::configuration::Configuration::default();
    vrchat_config.user_agent = Some(VRCHAT_UPDATER_USER_AGENT.to_string());
    vrchat_config.basic_auth = Some((config.vrchat_username.clone(), Some(config.vrchat_password.clone())));

    // Either re-use the cookie or authenticate on the first request.
    match authentication_api::get_current_user(&vrchat_config)
        .await
        .unwrap()
    {
        vrchatapi::models::EitherUserOrTwoFactor::CurrentUser(_me) => {
            // this case of an already authenticated cookie shouldn't happen, as we freshly create the config above
            true
        }
        vrchatapi::models::EitherUserOrTwoFactor::RequiresTwoFactorAuth(requires_auth) => {
            if requires_auth.requires_two_factor_auth.contains(&String::from("emailOtp"))
            {
                let code = read_user_input(&format!("Your account {} has received an Email with a 2FA code. Please enter it: ", config.vrchat_username));
                authentication_api::verify2_fa_email_code(&vrchat_config,TwoFactorEmailCode::new(code)).await?.verified
            } else {
                let code = read_user_input(&format!("Please enter your Authenticator 2FA code for the account {}:", config.vrchat_username));
                authentication_api::verify2_fa(&vrchat_config, TwoFactorAuthCode::new(code)).await?.verified
            }
        }
    };

    // Test, if the authentication is working
    let test_authentication = match authentication_api::get_current_user(&vrchat_config)
        .await
        .unwrap()
    {
        EitherUserOrTwoFactor::CurrentUser(user) => Ok(user.id),
        EitherUserOrTwoFactor::RequiresTwoFactorAuth(_) => Err(anyhow!("Cookie invalid for user {}", config.vrchat_username)),
    };

    test_authentication.map(|user_id| (vrchat_config, user_id))
}


fn generate_html(config: &Config, fronts: Vec<MemberContent>) -> String {
    let fronts_formatted = fronts
        .into_iter()
        .map(|m| -> String {
            format!("<div><img src=\"{}\" /><p>{}</p></div>",m.avatarUrl,html_escape::encode_text(&m.name))
        })
        .collect::<Vec<String>>()
        .join("\n");

    format!(r#"<html>
    <head>
        <title>{} - Fronting Status</title>
        <style>
            /* generated with ChatGPT o3 */
            /* --- layout container ------------------------------------ */
            body{{
                margin:0;
                padding:1rem;
                font-family:sans-serif;
                display:flex;
                flex-direction: column;
                gap:1rem;
            }}

            /* --- one card -------------------------------------------- */
            body>div {{
                flex:1 1 calc(25% - 1rem);   /* ≤4 cards per row */
                display:flex;
                align-items:center;
                gap:.75rem;
                padding:.75rem;
                background:#fff;
                border-radius:.5rem;
                box-shadow:0 2px 4px rgba(0,0,0,.08);
            }}

            /* --- avatar image ---------------------------------------- */
            body>div img {{
                width:10rem;
                height:10rem;           /* fixed square keeps things tidy */
                object-fit:cover;
                border-radius:50%;
            }}

            /* --- name ------------------------------------------------- */
            body>div p {{
                margin:0;
                font-size: 3rem;
                font-weight:600;
            }}

            /* --- phones & tablets ------------------------------------ */
            @media (max-width:800px) {{
                body>div {{flex:1 1 calc(50% - 1rem);}}   /* 2-across */
            }}
            @media (max-width:420px) {{
                body>div {{flex:1 1 100%;}}               /* stack */
            }}
        </style>
    </head>
    <body>
        {}
    </body>
</html>"#, html_escape::encode_text(&config.system_name), fronts_formatted)
}



fn read_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    input.trim().to_string()
}


async fn load_config() -> Result<Config> {
    eprintln!("Loading environment variables...");
    let serve_api = str::eq(&env::var("SERVE_API").expect("SERVE_API not set."), "true");
    eprintln!("SERVE_API is {}", serve_api);

    let sps_token = env::var("SPS_API_TOKEN").expect("SPS_API_TOKEN not set");

    let optional_vrchat_config = if serve_api { Ok("".to_string()) } else { Err("VRChat variables needs configuration.") };
    
    let vrchat_username = optional_vrchat_config.clone().or(env::var("VRCHAT_USERNAME")).expect("VRCHAT_USERNAME not set");
    let vrchat_password = optional_vrchat_config.clone().or(env::var("VRCHAT_PASSWORD")).expect("VRCHAT_PASSWORD not set");
    eprintln!("Credentials loaded. VRCHAT_USERNAME is {}", vrchat_username);

    let system_name = env::var("SYSTEM_PUBLIC_NAME").expect("SYSTEM_PUBLIC_NAME not set.");

    let vrchat_updater_prefix = env::var("VRCHAT_UPDATER_PREFIX").expect("VRCHAT_UPDATER_PREFIX not set.");
    let vrchat_updater_no_fronts = env::var("VRCHAT_UPDATER_NO_FRONTS").expect("VRCHAT_UPDATER_NO_FRONTS not set.");
    let vrchat_updater_truncate_names_to = env::var("VRCHAT_UPDATER_TRUNCATE_NAMES_TO")
        .expect("VRCHAT_UPDATER_TRUNCATE_NAMES_TO not set.")
        .parse::<usize>()
        .unwrap();

    let wait_seconds = env::var("SECONDS_BETWEEN_UPDATES")
        .expect("SECONDS_BETWEEN_UPDATES not set.")
        .parse::<u64>()
        .unwrap();
    
    let sps_base_url = env::var("SPS_API_BASE_URL").expect("SPS_API_BASE_URL not set.");
    eprintln!("Using SPS base URL: {}", sps_base_url);

    // Build HTTP client
    let client = Client::builder()
        .cookie_store(true)
        .build()
        .expect("Failed to build HTTP client");

    return Ok(Config{
        sps_token,
        vrchat_username,
        vrchat_password,
        vrchat_updater_prefix,
        vrchat_updater_no_fronts,
        vrchat_updater_truncate_names_to,
        sps_base_url,
        serve_api,
        system_name,
        wait_seconds,
        client,
    })
}


#[derive(Deserialize, Debug, Clone)]
struct FrontEntry {
    content: FrontEntryContent,
}

#[derive(Deserialize, Debug, Clone)]
struct FrontEntryContent {
    member: String, // member ID
    uid: String, // System ID
}

#[derive(Deserialize, Debug, Clone)]
struct Member {
    content: MemberContent,
    id: String, // member id
}

#[derive(Deserialize, Debug, Clone)]
#[allow(non_snake_case)] // VRChat JSON fields
struct MemberContent {
    name: String,
    avatarUrl: String,
}

#[derive(Debug, Clone)]
struct Config {
    sps_token: String,
    vrchat_username: String,
    vrchat_password: String,
    sps_base_url: String,
    client: Client,
    serve_api: bool,
    wait_seconds: u64,
    system_name: String,
    vrchat_updater_prefix: String,
    vrchat_updater_no_fronts: String,
    vrchat_updater_truncate_names_to: usize,
}
