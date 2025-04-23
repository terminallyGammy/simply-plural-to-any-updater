use serde::Deserialize;
use std::{env, fmt::Debug, thread, time, vec};
use time::{Duration};
use reqwest::{Client, Result};
use serde_json::json;
use tokio;
use chrono::{self, DateTime};

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
struct MemberContent {
    name: String,
}

#[derive(Debug, Clone)]
struct Config {
    sps_token: String,
    vrchat_username: String,
    vrchat_password: String,
    sps_base_url: String,
    vrchat_base_url: String,
    client: Client,
    run_once: bool,
}



async fn get_fronter_names(front_entries: Vec<FrontEntry>, config: &Config) -> Result<Vec<String>> {
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
    eprintln!("Received response (status: {})", members_response.status());

    let members: Vec<Member> = members_response.error_for_status()?.json().await?;

    let fronting_member_names: Vec<String> = members
        .into_iter()
        .filter(|m| front_uids.contains(&m.id))
        .map(|m| m.content.name)
        .collect();
    eprintln!("Fronting member names: {:?}", fronting_member_names);

    return Ok(fronting_member_names);
}   


async fn fetch_fronts_and_transfer_to_vrchat(config: &Config) -> Result<()> {
    // 1. Fetch current fronts from Simply Plural
    let fronts_url = format!("{}/fronters", &config.sps_base_url);
    eprintln!("Fetching fronts from SPS: {}", fronts_url);
    let fronts_response = config
        .client
        .get(&fronts_url)
        .header("Authorization", &config.sps_token)
        .send()
        .await?;
    eprintln!("Received response (status: {})", fronts_response.status());
    
    let front_entries: Vec<FrontEntry> = fronts_response
        .error_for_status()?
        .json()
        .await?;
    eprintln!("Parsed {} front entries.", front_entries.len());

    let fronter_ids: Vec<&String> = front_entries.iter().map(|e| &e.content.member).collect();
    eprintln!("Fronter IDs: {:?}", fronter_ids);

    let fronts: Vec<String> = if fronter_ids.is_empty() { vec![] } else { get_fronter_names(front_entries, config).await? };

    // Print members for reading from terminal
    fronts.into_iter().for_each(|name| println!("{}",name));


    // // Format status as "F: <fronter1>, <fronter2>, ..."
    // let status_desc = if front_names.is_empty() {
    //     eprintln!("No fronts found.");
    //     "F: none?".to_string()
    // } else {
    //     let desc = format!("F: {}", front_names.join(", "));
    //     eprintln!("Formatted statusDescription: {}", desc);
    //     desc
    // };

    // // 2. Authenticate with VRChat
    // let auth_url = format!("{}/auth/user", vr_base);
    // eprintln!("Authenticating with VRChat: {}", auth_url);
    // let auth_response = client
    //     .get(&auth_url)
    //     .basic_auth(&vr_username, Some(&vr_password))
    //     .send()
    //     .await?;
    // eprintln!("Authenticated (status: {})", auth_response.status());
    // let auth_json: serde_json::Value = auth_response
    //     .error_for_status()?
    //     .json()
    //     .await?;
    // let user_id = auth_json["id"].as_str().expect("Missing user ID");
    // eprintln!("Retrieved user ID: {}", user_id);

    // // 3. Update VRChat status
    // let update_url = format!("{}/users/{}", vr_base, user_id);
    // eprintln!("Updating VRChat status at: {}", update_url);
    // let update_payload = json!({
    //     "status": "active",
    //     "statusDescription": status_desc,
    // });
    // eprintln!("Payload: {}", update_payload);
    // let update_response = client
    //     .put(&update_url)
    //     .basic_auth(&vr_username, Some(&vr_password))
    //     .json(&update_payload)
    //     .send()
    //     .await?;
    // eprintln!("Update response status: {}", update_response.status());

    // eprintln!("VRChat status updated successfully.");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    eprintln!("Starting VRChat SPS status updater...");

    // Load configuration
    let config = load_config().await?;

    loop {
        eprintln!("\n\n======================= UTC {}",chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"));
        fetch_fronts_and_transfer_to_vrchat(&config).await?;
        if config.run_once { break; }
        thread::sleep(Duration::from_secs(60));
    }

    Ok(())
}

async fn load_config() -> Result<Config> {
    eprintln!("Loading environment variables...");
    let sps_token = env::var("SPS_API_TOKEN").expect("SPS_API_TOKEN not set");
    
    let vrchat_username = env::var("VRCHAT_USERNAME").expect("VRCHAT_USERNAME not set");
    let vrchat_password = env::var("VRCHAT_PASSWORD").expect("VRCHAT_PASSWORD not set");
    
    eprintln!("Credentials loaded. VRCHAT_USERNAME is {}", vrchat_username);

    let run_once = env::var("RUN_ONCE").expect("RUN_ONCE not set.");
    
    let sps_base_url = env::var("SPS_API_BASE_URL")
    .unwrap_or_else(|_| "https://api.apparyllis.com/v1".to_string());
    eprintln!("Using SPS base URL: {}", sps_base_url);
    let vrchat_base_url = env::var("VRCHAT_API_BASE_URL")
    .unwrap_or_else(|_| "https://api.vrchat.cloud/api/1".to_string());
    eprintln!("Using VRChat base URL: {}", vrchat_base_url);

    // Build HTTP client
    let client = Client::builder()
        .cookie_store(true)
        .build()
        .expect("Failed to build HTTP client");

    return Ok(Config{
        sps_token,
        vrchat_username,
        vrchat_password,
        sps_base_url,
        vrchat_base_url,
        run_once: str::eq(&run_once, "true"),
        client,
    })
}
