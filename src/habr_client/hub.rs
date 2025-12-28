use std::collections::HashMap;

use reqwest::Error;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Hub {
    pub id: String,
    pub alias: String,
    #[serde(alias = "titleHtml")]
    pub title: String,
    #[serde(rename(deserialize = "descriptionHtml"))]
    pub description_html: String,
    #[serde(rename(deserialize = "commonTags"))]
    pub common_tags: Vec<String>,
    #[serde(rename(deserialize = "imageUrl"))]
    pub image_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct HubsResponse {
    #[serde(rename(deserialize = "pagesCount"))]
    pages_count: usize,
    #[serde(rename(deserialize = "hubIds"))]
    hub_ids: Vec<serde_json::Value>,
    #[serde(rename(deserialize = "hubRefs"))]
    hub_refs: HashMap<String, Hub>,
}

pub async fn get_hubs(page: u8, search_text: String) -> Result<(Vec<Hub>, usize), Error> {
    let url = if search_text.is_empty() {
        "https://habr.com/kek/v2/hubs"
    } else {
        "https://habr.com/kek/v2/hubs/search"
    };

    let resp = reqwest::Client::new()
        .get(url)
        .header("Cookie", "fl=ru; hl=ru;")
        .query(&[
            ("q", search_text.as_str()),
            ("page", page.to_string().as_str()),
            ("fl", "ru"),
            ("hl", "ru"),
            // ("perPage", "30"),
        ])
        .send()
        .await?;

    let resp_parsed: HubsResponse = serde_json::from_slice(&resp.bytes().await.unwrap())
        .expect("[!] Error with response parsing");

    let mut hubs: Vec<Hub> = resp_parsed.hub_refs.into_values().collect();

    hubs.sort_by(|f, s| f.title.cmp(&s.title));
    hubs.iter_mut().for_each(|h| h.title = super::html_parse::extract_text_from_html(&h.title) );

    Ok((hubs, resp_parsed.pages_count))
}
