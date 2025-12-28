use std::str::FromStr;

use chrono::{DateTime, Local};
use reqwest::{header, Client, Error, Method, RequestBuilder};

pub mod article;
pub mod hub;
pub mod html_parse;

use article::{ArticleContent, ArticleData, ArticleResponse, ArticlesResponse, ArticlesListSorting, ArticlesListFilter, ArticlesSearchSorting};
use html_parse::{extract_content_from_html, extract_text_from_html};

type PagesCount = usize;

#[derive(Clone)]
pub struct HabrClient {
    client: Client,
}

impl HabrClient {
    pub fn new() -> Self {
        let mut default_headers = header::HeaderMap::new();
        default_headers.insert(header::COOKIE, "fl=ru; hl=ru;".parse().unwrap());

        let client = reqwest::Client::builder()
            .default_headers(default_headers)
            .build()
            .unwrap();

        Self { client }
    }

    fn setup_request(&self, method: Method, url: &str) -> RequestBuilder {
        self.client
            .request(method, url)
            .query(&[("fl", "ru"), ("hl", "ru")])
    }

    pub async fn get_article_details(
        &self,
        article_id: &str,
    ) -> Result<(String, Vec<ArticleContent>), Error> {
        let url = format!("https://habr.com/kek/v2/articles/{}", article_id);
        let resp = self.setup_request(Method::GET, url.as_str()).send().await?;

        let resp_parsed: ArticleResponse = serde_json::from_slice(&resp.bytes().await.unwrap())
            .expect("[!] Error with response parsing");

        Ok((
            resp_parsed.title,
            tokio::spawn(extract_content_from_html(resp_parsed.text))
                .await
                .unwrap(),
        ))
    }

    pub async fn search_articles(
        &self,
        search_query: &str,
        sort: ArticlesSearchSorting,
        page: u8,
    ) -> Result<(Vec<ArticleData>, PagesCount), Error>
    {
        let resp = self
            .setup_request(Method::GET, "https://habr.com/kek/v2/articles/")
            .query(&[
                ("page", page.to_string().as_str()),
                ("query", search_query),
                ("order", &sort.to_string()),
                ("perPage", "20"),
            ])
            .send()
            .await?;

        let response_bytes = resp.bytes().await.unwrap();
        // println!("{}", String::from_utf8_lossy(&response_bytes));
        let resp_parsed: ArticlesResponse = serde_json::from_slice(&response_bytes)
            .expect("[!] Error with response parsing");

        let mut articles: Vec<ArticleData> = resp_parsed
            .articles
            .into_values()
            .map(|a| {
                let published_at: DateTime<Local> = DateTime::from_str(&a.published_at).unwrap();

                ArticleData {
                    id: a.id.into(),
                    title: extract_text_from_html(a.title.trim()),
                    author: a.author.map_or("".to_string(), |a| a.alias),
                    reading_time: a.reading_time,
                    published_at: format!("{}", published_at.format("%d.%m.%Y %H:%M")),
                    tags: a.tags.into_iter().map(|t| t.title).collect(),
                    complexity: a.complexity.unwrap_or(String::new()),
                    image_url: a.lead_data.image_url.unwrap_or("".to_string()),
                    score: a.statistics.score,
                }
            })
            .collect();

        match sort {
            ArticlesSearchSorting::Rating => {
                articles.sort_by(|a, b| a.score.cmp(&b.score));
            },
            _ => {}
        }

        Ok((articles, resp_parsed.pages_count))
    }

    pub async fn get_articles(
        &self,
        hub: String,
        sorting: ArticlesListSorting,
        filter: ArticlesListFilter,
        page: u8,
    ) -> Result<(Vec<ArticleData>, PagesCount), Error> {
        let filter_params: (&str, String) = match filter {
            ArticlesListFilter::ByDate(date) => ("period", date.to_string()),
            ArticlesListFilter::ByRating(rating) => ("score", rating.map_or(String::new(), |s| s.to_string()))
        };
        let resp = self
            .setup_request(Method::GET, "https://habr.com/kek/v2/articles/")
            .query(&[
                ("page", page.to_string()),
                ("hub", hub.to_string()),
                ("sort", if hub.is_empty() {sorting.to_string()} else {String::from("all")}),
                ("perPage", String::from("20")),
                filter_params
            ])
            .send()
            .await?;

        let response_bytes = resp.bytes().await.unwrap();
        // println!("{}", String::from_utf8_lossy(&response_bytes));
        let resp_parsed: ArticlesResponse = serde_json::from_slice(&response_bytes)
            .expect("[!] Error with response parsing");

        let mut articles: Vec<ArticleData> = resp_parsed
            .articles
            .into_values()
            .map(|a| {
                let published_at: DateTime<Local> = DateTime::from_str(&a.published_at).unwrap();

                ArticleData {
                    id: a.id.into(),
                    title: extract_text_from_html(a.title.trim()),
                    author: a.author.map_or("".to_string(), |a| a.alias),
                    reading_time: a.reading_time,
                    published_at: format!("{}", published_at.format("%d.%m.%Y %H:%M")),
                    tags: a.tags.into_iter().map(|t| t.title).collect(),
                    complexity: a.complexity.unwrap_or(String::new()),
                    image_url: a.lead_data.image_url.unwrap_or("".to_string()),
                    score: a.statistics.score,
                }
            })
            .collect();

        match sorting {
            ArticlesListSorting::Best => {
                articles.sort_by(|a, b| b.score.cmp(&a.score));
            },
            _ => {}
        }

        Ok((articles, resp_parsed.pages_count))
    }
}
