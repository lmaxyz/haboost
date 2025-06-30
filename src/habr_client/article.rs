use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json;

use super::TypedText;


#[derive(Serialize, Deserialize, Debug)]
pub struct LeadData {
    #[serde(alias = "textHtml")]
    pub description: String,
    #[serde(alias = "imageUrl")]
    pub image_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tag {
    #[serde(alias = "titleHtml")]
    pub title: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Author {
    pub(crate) id: String,
    pub(crate) alias: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArticlePreviewResponse {
    pub id: String,
    #[serde(alias = "timePublished")]
    pub published_at: String,
    #[serde(alias = "titleHtml")]
    pub title: String,
    #[serde(rename(deserialize = "leadData"))]
    pub lead_data: LeadData,
    pub(crate) tags: Vec<Tag>,
    pub(crate) complexity: Option<String>,
    #[serde(alias = "readingTime")]
    pub(crate) reading_time: usize,
    pub(crate) author: Option<Author>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArticlesResponse {
    #[serde(rename(deserialize = "pagesCount"))]
    pub(crate) pages_count: usize,
    #[serde(rename(deserialize = "publicationIds"))]
    pub(crate) article_ids: Vec<serde_json::Value>,
    #[serde(rename(deserialize = "publicationRefs"))]
    pub(crate) articles: HashMap<String, ArticlePreviewResponse>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ArticleResponse {
    #[serde(alias = "titleHtml")]
    pub title: String,
    #[serde(alias = "textHtml")]
    pub text: String,
}


#[derive(Clone, Debug)]
pub struct ArticleData {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) tags: Vec<String>,
    pub(crate) complexity: String,
    pub(crate) author: String,
    pub(crate) published_at: String,
    pub(crate) reading_time: usize,
    pub image_url: String,
}


pub enum ArticleContent {
    Image(String),
    Header(u8, String),
    Paragraph(Vec<TypedText>),
    Code {
        lang: String,
        content: String
    }
}
