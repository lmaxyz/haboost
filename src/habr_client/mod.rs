use chrono;
use reqwest::{header, Client, Error, Method, RequestBuilder};
use scraper::{ElementRef, Html};
use std::str::FromStr;

pub mod article;
pub mod hub;

use article::{ArticleContent, ArticleData, ArticleResponse, ArticlesResponse};

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
                ("perPage", "20"),
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
                let published_at: chrono::DateTime<chrono::Local> =
                    chrono::DateTime::from_str(&a.published_at).unwrap();

                ArticleData {
                    id: a.id.into(),
                    title: a.title.trim().into(),
                    author: a.author.map_or("".to_string(), |a| a.alias),
                    reading_time: a.reading_time,
                    published_at: format!("{}", published_at.format("%d.%m.%Y %H:%M")),
                    _tags: a.tags.into_iter().map(|t| t.title).collect(),
                    complexity: a.complexity.unwrap_or(String::new()),
                    image_url: a.lead_data.image_url.unwrap_or("".to_string()),
                }
            })
            .collect();

        Ok((articles, resp_parsed.pages_count))
    }
}

fn extract_text_from_html(input: &str) -> String {
    let html = Html::parse_fragment(&input);
    // if let Some(first_child) = html.root_element().first_child() {
    //     if let Some(txt) = first_child.value().as_text() {
    //         return TypedText::Common(txt.to_string())
    //     } else if let Some(elem) = first_child.value().as_element() {
    //         match elem.name() {
    //             "em" => return TypedText::Italic(first_child.first_child().map_or("".to_string(), |item| item.value().as_text().unwrap().to_string())),
    //             _ => {}
    //         }
    //     }
    // }
    get_element_text(&html.root_element())
}

async fn extract_content_from_html(text: String) -> Vec<ArticleContent> {
    let html = Html::parse_fragment(&text);
    // for node in html.tree.values().into_iter() {
    //     println!("Node: {:?}", node);
    //     // if let Some(el) = node.as_element() {
    //     //     println!("Node: {:?}", el);
    //     // }
    // }
    parse_content_recursively(html.root_element())
}

fn parse_content_recursively<'a>(element: ElementRef<'a>) -> Vec<ArticleContent> {
    let mut res = Vec::new();
    if let Ok(content) = element.try_into() {
        res.push(content);
    }
    for inner_elem in element.child_elements() {
        res.extend(parse_content_recursively(inner_elem))
    }
    res
}

fn extract_paragraph_content<'a>(element: &ElementRef<'a>) -> Vec<TypedText> {
    if element.value().name() == "p" && element.has_children() {
        return element.children().filter_map(|child| {
            if let Some(txt) = child.value().as_text() {
                return Some(TypedText::Common(txt.to_string()))
            }
            if let Some(elem) = child.value().as_element() {
                match elem.name() {
                    "code" => {
                        if let Some(code_child) = child.first_child() {
                            if let Some(code_text) = code_child.value().as_text() {
                                return Some(TypedText::Code(code_text.to_string()))
                            }
                            println!("Code block with not a text child: {:?}", code_child);
                        }
                        println!("Code block without child")
                    },
                    "a" => {
                        let url = elem.attr("href").unwrap().to_string();
                        let value = if let Some(link_child) = child.first_child() {
                            if let Some(link_text) = link_child.value().as_text() {
                                link_text.to_string()
                            } else {
                                url.clone()
                            }
                        } else {
                            url.clone()
                        };
                        let link = TypedText::Link {
                            url,
                            value
                        };
                        println!("Link inside paragraph: {:?}", link);
                        return Some(link)
                    },
                    "i" => {
                        if let Some(text_item) = child.first_child() {
                            if let Some(text) = text_item.value().as_text() {
                                return Some(TypedText::Italic(text.to_string()))
                            }
                        }
                    },
                    "strong" => {
                        if let Some(text_item) = child.first_child() {
                            if let Some(text) = text_item.value().as_text() {
                                return Some(TypedText::Strong(text.to_string()))
                            }
                        }
                    }
                    _tag_name => {}
                }
            }
            None
        }).collect()
    }
    Vec::new()
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
            "p" => Ok(ArticleContent::Paragraph(extract_paragraph_content(element))),
            "h2" => Ok(ArticleContent::Header(2, get_element_text(element))),
            "h3" => Ok(ArticleContent::Header(3, get_element_text(element))),
            "h4" => Ok(ArticleContent::Header(4, get_element_text(element))),
            "code" => {
                if element.parent().unwrap().value().as_element().unwrap().name() == "p" {
                    return Err(())
                }
                Ok(ArticleContent::Code{
                    lang: element.attr("class").unwrap_or("").to_string(),
                    content: get_element_text(element)
                })
            },
            // "a" => {
            //     Ok(ArticleContent::Text(
            //     get_element_text(element),
            //     TextType::Link(element.attr("href").unwrap_or("").to_string()),
            // ))},
            _tag @ _ => {
                println!("[!] Unsupported tag: {}", _tag);
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


#[derive(Debug, Clone)]
pub enum TypedText {
    Common(String),
    Code(String),
    Link {
        url: String,
        value: String
    },
    Italic(String),
    Strong(String),
}
