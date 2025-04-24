use serde::Deserialize;
use std::{env, fmt::Debug, thread, time, vec};
use time::{Duration};
use reqwest::{Client, Result};
use serde_json::json;
use tokio;
use chrono;


#[tokio::main]
async fn main() -> Result<()> {
    eprintln!("Starting VRChat SPS status updater...");

    let config = load_config().await?;

    if config.serve_api {
        // todo. start server and on each request:
        let fronts = fetch_fronts(&config).await?;
        let html = generate_html(&config, fronts);

        println!("{}",html);

        Ok(())
    }
    else {
        update_vrchat_status_fronts_loop(&config).await
    }
}



async fn update_vrchat_status_fronts_loop(config: &Config) -> Result<()>{
    loop {
        eprintln!("\n\n======================= UTC {}",chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"));

        let fronts = fetch_fronts(&config).await?;
        
        update_fronts_in_vrchat_status(&config, fronts).await?;
        
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




async fn update_fronts_in_vrchat_status(config: &Config, fronts: Vec<MemberContent>) -> Result<()> {
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





fn generate_html(config: &Config, fronts: Vec<MemberContent>) -> String {
    let fronts_formatted = fronts
        .into_iter()
        .map(|m| -> String { format!("<p>{}</p><p><img src=\"{}\" /></p>",html_escape::encode_text(&m.name),m.avatarUrl)})
        .collect::<Vec<String>>()
        .join("<hr />");

    format!(r#"<html>
    <head>
        <title>{} - Fronting Status</title>
    </head>
    <body>
        {}
    </body>
</html>"#, html_escape::encode_text(&config.system_name), fronts_formatted)
}




async fn load_config() -> Result<Config> {
    eprintln!("Loading environment variables...");
    let sps_token = env::var("SPS_API_TOKEN").expect("SPS_API_TOKEN not set");
    
    let vrchat_username = env::var("VRCHAT_USERNAME").expect("VRCHAT_USERNAME not set");
    let vrchat_password = env::var("VRCHAT_PASSWORD").expect("VRCHAT_PASSWORD not set");
    
    eprintln!("Credentials loaded. VRCHAT_USERNAME is {}", vrchat_username);

    let serve_api = env::var("SERVE_API").expect("SERVE_API not set.");

    let system_name = env::var("SYSTEM_PUBLIC_NAME").expect("SYSTEM_PUBLIC_NAME not set.");
    

    let wait_seconds = env::var("SECONDS_BETWEEN_UPDATES")
        .expect("SECONDS_BETWEEN_UPDATES not set.")
        .parse::<u64>()
        .unwrap();
    
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
        serve_api: str::eq(&serve_api, "true"),
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
    vrchat_base_url: String,
    client: Client,
    serve_api: bool,
    wait_seconds: u64,
    system_name: String,
}
