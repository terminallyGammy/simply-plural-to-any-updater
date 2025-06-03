use serde::Deserialize;
use anyhow::Result;

use crate::config::Config;


pub(crate) async fn fetch_fronts(config: &Config) -> Result<Vec<MemberContent>> {    
    let front_entries =simply_plural_http_request_get_fronters(config).await?;

    let fronts = enrich_fronter_ids_with_member_info(front_entries, config).await?;

    Ok(fronts)
}


async fn enrich_fronter_ids_with_member_info(front_entries: Vec<FrontEntry>, config: &Config) -> Result<Vec<MemberContent>> {
    if front_entries.is_empty() {
        return Ok(vec![])
    }

    let system_id = &front_entries[0].content.uid;
    let all_members = simply_plural_http_get_members(config, system_id).await?;
    
    let fronters: Vec<String>  = front_entries.iter().map(|e| e.content.member.clone()).collect();
    let enriched_fronting_members: Vec<MemberContent> = all_members
        .into_iter()
        .filter(|m| fronters.contains(&m.id))
        .map(|m| {
            eprintln!("Fronting member: {:?}",m.content);
            m.content
        })
        .collect();

    return Ok(enriched_fronting_members);
}

async fn simply_plural_http_request_get_fronters(config: &Config) -> Result<Vec<FrontEntry>> {
    eprintln!("Fetching fronts from SimplyPlural...");
    let fronts_url = format!("{}/fronters", &config.simply_plural_base_url);
    let result = config
        .client
        .get(&fronts_url)
        .header("Authorization", &config.simply_plural_token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    
    Ok(result)
}

async fn simply_plural_http_get_members(config: &Config, system_id: &String) -> Result<Vec<Member>> {
    eprintln!("Fetching all members from SimplyPlural..");
    let fronts_url = format!("{}/members/{}", &config.simply_plural_base_url, system_id);
    let result = config
        .client
        .get(&fronts_url)
        .header("Authorization", &config.simply_plural_token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(result)
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

impl MemberContent {
    pub(crate) fn preferred_vrchat_status_name(&self) -> &String {
        if self.info.vrchat_status_name.is_empty() { &self.name } else { &self.info.vrchat_status_name }
    }
}