use serde::Deserialize;
use anyhow::Result;

use crate::config::Config;


pub(crate) async fn fetch_fronts(config: &Config) -> Result<Vec<MemberContent>> {
    let fronts_url = format!("{}/fronters", &config.sps_base_url);
    eprintln!("Fetching fronts from SPS: {}", fronts_url);
    let fronts_response = config
        .client
        .get(&fronts_url)
        .header("Authorization", &config.sps_token)
        .send()
        .await?;
    
    let front_entries: Vec<FrontEntry> = fronts_response
        .error_for_status()?
        .json()
        .await?;

    let fronter_ids: Vec<&String> = front_entries.iter().map(|e| &e.content.member).collect();
    eprintln!("Fronter IDs: {:?}", fronter_ids);

    let fronts: Vec<MemberContent> = if fronter_ids.is_empty() { vec![] } else { enrich_fronter_ids_with_member_info(front_entries, config).await? };

    Ok(fronts)
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



#[derive(Deserialize, Debug, Clone)]
pub struct FrontEntry {
    pub content: FrontEntryContent,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FrontEntryContent {
    pub member: String, // member ID
    pub uid: String,    // System ID
}

#[derive(Deserialize, Debug, Clone)]
pub struct Member {
    pub content: MemberContent,
    pub id: String, // member id
}

#[derive(Deserialize, Debug, Clone)]
pub struct MemberContent {
    pub name: String,
    
    #[serde(rename = "avatarUrl")]
    #[serde(default)]
    pub avatar_url: String,
    
    #[serde(default)]
    pub info: MemberContentInfo,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct MemberContentInfo {
    // this is the id of the custom field "VRChat Status Name"
    #[serde(rename = "683b8c2b7a5026a429000000")]
    pub vrchat_status_name: String,
}