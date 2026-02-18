use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json;

use super::html_parse::TypedText;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_article_response() {
        let json = r#"{
            "titleHtml": "<h1>Test Article</h1>",
            "textHtml": "<p>Article content</p>"
        }"#;

        let response: ArticleResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.title, "<h1>Test Article</h1>");
        assert_eq!(response.text, "<p>Article content</p>");
    }

    #[test]
    fn test_parse_articles_list_response() {
        let json = r#"{
            "pagesCount": 5,
            "publicationIds": ["123", "456"],
            "publicationRefs": {
                "123": {
                    "id": "123",
                    "timePublished": "2026-01-25T08:00:00Z",
                    "titleHtml": "<h2>Article 1</h2>",
                    "leadData": {
                        "textHtml": "Description",
                        "imageUrl": "https://habr.com/image.png"
                    },
                    "tags": [{"titleHtml": "Rust"}],
                    "complexity": "Easy",
                    "readingTime": 10,
                    "author": {
                        "id": "1",
                        "alias": "Author1",
                        "avatarUrl": null
                    },
                    "statistics": {
                        "commentsCount": 42,
                        "readingCount": 1000,
                        "score": 50
                    }
                }
            }
        }"#;

        let response: ArticlesResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.pages_count, 5);
        assert_eq!(response.article_ids.len(), 2);

        let article = response.articles.get("123").unwrap();
        assert_eq!(article.id, "123");
        assert_eq!(article.reading_time, 10);
        assert_eq!(article.statistics.comments_count, 42);
        assert_eq!(article.statistics.score, 50);
    }
}

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
    pub score: isize,
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
    pub tags: Vec<Tag>,
    pub complexity: Option<String>,
    #[serde(alias = "readingTime")]
    pub reading_time: usize,
    pub author: Option<Author>,
    pub statistics: Statistics,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArticlesResponse {
    #[serde(rename(deserialize = "pagesCount"))]
    pub pages_count: usize,
    #[serde(rename(deserialize = "publicationIds"))]
    pub article_ids: Vec<serde_json::Value>,
    #[serde(rename(deserialize = "publicationRefs"))]
    pub articles: HashMap<String, ArticlePreviewResponse>,
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
    pub comments_count: usize,
}

#[derive(Clone, Debug)]
pub enum ArticleContent {
    Image(String),
    Header(u8, String),
    Paragraph(Vec<TypedText>),
    Code { lang: String, content: String },
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
    ByDate(DateFilter),
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
