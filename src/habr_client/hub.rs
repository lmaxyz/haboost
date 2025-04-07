use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json;
use reqwest::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct HubItem {
    id: String,
    alias: String,
    #[serde(alias = "titleHtml")]
    pub title: String,
    #[serde(rename(deserialize = "descriptionHtml"))]
    pub description_html: String,
    #[serde(rename(deserialize = "commonTags"))]
    common_tags: Vec<String>,
    #[serde(rename(deserialize = "imageUrl"))]
    pub image_url: String
}

#[derive(Serialize, Deserialize, Debug)]
struct HubsResponse {
    #[serde(rename(deserialize = "pagesCount"))]
    pages_count: usize,
    #[serde(rename(deserialize = "hubIds"))]
    hub_ids: Vec<serde_json::Value>,
    #[serde(rename(deserialize = "hubRefs"))]
    hub_refs: HashMap<String, HubItem>,
}

// impl Into<HubData> for HubItem  {
//     fn into(self) -> HubData {
//         let default_img = DynamicImage::new(96, 96, image::ColorType::Rgba8);
//         let img_buf = SharedPixelBuffer::clone_from_slice(default_img.as_bytes(), default_img.width(), default_img.height());
//         let image = Image::from_rgb8(img_buf);
//         HubData {
//             id: self.alias.into(),
//             title: self.title.into(),
//             description: self.description_html.into(),
//             image_url: self.image_url.into(),
//             image
//         }
//     }
// }

pub async fn get_hubs(page: u8) -> Result<(Vec<HubItem>, usize), Error> {
    let resp = reqwest::Client::new()
        .get("https://habr.com/kek/v2/hubs/")
        .header("Cookie", "fl=ru; hl=ru;")
        .query(&[
            ("page", page.to_string().as_str()),
            ("fl", "ru"),
            ("hl", "ru"),
            ("perPage", "30")
        ])
        .send().await?;

    let resp_parsed: HubsResponse = serde_json::from_slice(&resp.bytes().await.unwrap()).expect("[!] Error with response parsing");

    let mut hubs: Vec<HubItem> = resp_parsed.hub_refs.into_values().collect();

    hubs.sort_by(|f, s| {
        f.title.cmp(&s.title)
    });

    Ok((hubs, resp_parsed.pages_count))
}
