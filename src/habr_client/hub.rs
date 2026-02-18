use std::collections::HashMap;

use reqwest::Error;
use serde::{Deserialize, Serialize};
use serde_json;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hub_response() {
        let json = r#"{
            "pagesCount": 1,
            "hubIds": [],
            "hubRefs": {
                "ru_rust": {
                    "id": "rust",
                    "alias": "ru_rust",
                    "titleHtml": "<h2>Rust</h2>",
                    "descriptionHtml": "Rust programming language",
                    "commonTags": ["programming"],
                    "imageUrl": "https://habr.com/hub/rust.png",
                    "statistics": {
                        "subscribersCount": 50000,
                        "rating": 100.2
                    }
                }
            }
        }"#;

        let response: HubsResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.pages_count, 1);

        let hub = response.hub_refs.get("ru_rust").unwrap();
        assert_eq!(hub.alias, "ru_rust");
        assert_eq!(hub.statistics.subscribers_count, 50000);
    }
}

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
    pub statistics: HubStatistics,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HubStatistics {
    #[serde(rename(deserialize = "subscribersCount"))]
    pub subscribers_count: usize,
    pub rating: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HubsResponse {
    #[serde(rename(deserialize = "pagesCount"))]
    pub pages_count: usize,
    #[serde(rename(deserialize = "hubIds"))]
    pub hub_ids: Vec<serde_json::Value>,
    #[serde(rename(deserialize = "hubRefs"))]
    pub hub_refs: HashMap<String, Hub>,
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
    hubs.iter_mut()
        .for_each(|h| h.title = super::html_parse::extract_text_from_html(&h.title));

    Ok((hubs, resp_parsed.pages_count))
}
