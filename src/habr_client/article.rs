use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json;

use super::html_parse::TypedText;


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
    #[serde(alias = "avatarUrl")]
    pub(crate) avatar_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Statistics {
    #[serde(alias = "commentsCount")]
    pub comments_count: usize,
    #[serde(alias = "readingCount")]
    pub reading_count: usize,
    pub score: isize
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
    pub(crate) statistics: Statistics,
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
    #[allow(dead_code)]
    pub(crate) tags: Vec<String>,
    pub(crate) complexity: String,
    pub(crate) author: String,
    pub(crate) published_at: String,
    pub(crate) reading_time: usize,
    pub image_url: String,
    pub score: isize,
}

#[derive(Clone, Debug)]
pub enum ArticleContent {
    Image(String),
    Header(u8, String),
    Paragraph(Vec<TypedText>),
    Code {
        lang: String,
        content: String
    },
    Blockquote(String),
    Text(TypedText),
    UnorderedList(Vec<ArticleContent>),
    OrderedList(Vec<ArticleContent>),
    BR,
}

type RatingFilter = Option<usize>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArticlesListSorting {
    Newest,
    Best,
}

impl Default for ArticlesListSorting {
    fn default() -> Self {
        ArticlesListSorting::Newest
    }
}

impl ArticlesListSorting {
    pub fn to_string(&self) -> String {
        match self {
            ArticlesListSorting::Best => "date".to_string(),
            ArticlesListSorting::Newest => "rating".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArticlesListFilter {
    ByRating(RatingFilter),
    ByDate(DateFilter)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DateFilter {
    Daily,
    Weekly,
    Monthly,
    Yearly,
    AllTime,
}

impl DateFilter {
    pub fn to_string(&self) -> String {
        match self {
            DateFilter::Daily => "daily".to_string(),
            DateFilter::Weekly => "weekly".to_string(),
            DateFilter::Monthly => "monthly".to_string(),
            DateFilter::Yearly => "yearly".to_string(),
            DateFilter::AllTime => "alltime".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComplexityFilter {
    Easy,
    Medium,
    Hard,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArticlesSearchSorting {
    Relevance,
    Date,
    Rating,
}

impl ArticlesSearchSorting {
    pub fn to_string(&self) -> String {
        match self {
            ArticlesSearchSorting::Relevance => "relevance".to_string(),
            ArticlesSearchSorting::Date => "date".to_string(),
            ArticlesSearchSorting::Rating => "rating".to_string(),
        }
    }
}
