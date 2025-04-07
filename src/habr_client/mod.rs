use chrono;
use image::DynamicImage;
use reqwest::{header, Client, Error, Method, RequestBuilder};
use scraper::{ElementRef, Html};
use std::str::FromStr;

pub mod article;
pub mod hub;

use article::{ArticleContent, ArticleData, ArticleResponse, ArticlesResponse, TextType};

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

    pub async fn get_articles(
        &self,
        hub: &str,
        page: u8,
    ) -> Result<(Vec<ArticleData>, PagesCount), Error> {
        let resp = self
            .setup_request(Method::GET, "https://habr.com/kek/v2/articles/")
            .query(&[
                ("page", page.to_string().as_str()),
                ("hub", hub),
                ("sort", "all"),
                ("perPage", "15"),
            ])
            .send()
            .await?;

        let resp_parsed: ArticlesResponse = serde_json::from_slice(&resp.bytes().await.unwrap())
            .expect("[!] Error with response parsing");
        // println!("{:#?}", resp_parsed);
        let articles = resp_parsed
            .articles
            .into_values()
            .map(|a| {
                let image = DynamicImage::new_rgb8(480, 280);
                let published_at: chrono::DateTime<chrono::Local> =
                    chrono::DateTime::from_str(&a.published_at).unwrap();

                ArticleData {
                    id: a.id.into(),
                    title: a.title.trim().into(),
                    author: a.author.map_or("".to_string(), |a| a.alias),
                    reading_time: a.reading_time,
                    published_at: format!("{}", published_at.format("%d.%m.%Y %H:%M")),
                    tags: a.tags.into_iter().map(|t| t.title).collect(),
                    complexity: a.complexity.unwrap_or(String::new()),
                    image_url: a.lead_data.image_url.unwrap_or("".to_string()),
                    image,
                }
            })
            .collect();

        Ok((articles, resp_parsed.pages_count))
    }
}

async fn extract_content_from_html(text: String) -> Vec<ArticleContent> {
    let html = Html::parse_fragment(&text);
    parse_content_recursively(html.root_element())
}

fn parse_content_recursively<'a>(element: ElementRef<'a>) -> Vec<ArticleContent> {
    let mut res = Vec::new();
    let parsed_content = element.try_into().ok();
    if let Some(content) = parsed_content {
        res.push(content);
    }
    for inner_elem in element.child_elements() {
        res.extend(parse_content_recursively(inner_elem))
    }
    res
}

fn get_element_text<'a>(element: &ElementRef<'a>) -> String {
    element
        .text()
        .map(|txt| txt.trim())
        .collect::<Vec<&str>>()
        .join(" ")
}

impl TryFrom<&ElementRef<'_>> for ArticleContent {
    type Error = ();

    fn try_from(element: &ElementRef<'_>) -> Result<Self, Self::Error> {
        match element.value().name() {
            "img" => {
                if let Some(img_src) = element.attr("src") {
                    Ok(ArticleContent::Image(img_src.to_string()))
                } else {
                    Err(())
                }
            }
            "p" => Ok(ArticleContent::Text(
                get_element_text(element),
                TextType::Common,
            )),
            "h2" => Ok(ArticleContent::Text(
                get_element_text(element),
                TextType::Header(2),
            )),
            "h3" => Ok(ArticleContent::Text(
                get_element_text(element),
                TextType::Header(3),
            )),
            "h4" => Ok(ArticleContent::Text(
                get_element_text(element),
                TextType::Header(4),
            )),
            _tag @ _ => {
                // println!("[!] Unsupported tag: {}", tag);
                Err(())
            }
        }
    }
}

impl TryFrom<ElementRef<'_>> for ArticleContent {
    type Error = ();

    fn try_from(element: ElementRef<'_>) -> Result<Self, Self::Error> {
        (&element).try_into()
    }
}
