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
                    tags: a.tags.into_iter().map(|t| t.title).collect(),
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
    get_element_text(&html.root_element())
}

async fn extract_content_from_html(text: String) -> Vec<ArticleContent> {
    let html = Html::parse_fragment(&text);

    let mut res = Vec::new();
    if html.root_element().children().count() == 1 {
        let parsed = parse_recursively(&ElementRef::wrap(html.root_element().first_child().unwrap()).unwrap());
        res.extend(parsed);
    } else {

    }

    res
}

fn trim_first(index: usize, text: &str) -> String {
    if index == 0 {
        text.trim_start().to_string()
    } else {
        text.to_string()
    }
}

fn extract_paragraph_content<'a>(element: &ElementRef<'a>) -> Vec<TypedText> {
    element.children().enumerate().filter_map(|(index, p_child)| {
        if let Some(txt) = p_child.value().as_text() {
            return Some(TypedText::Common(trim_first(index, txt)))
        }
        if let Some(elem) = p_child.value().as_element() {
            if let Some(inner_child) = p_child.first_child() {
                if elem.name() == "a" {
                    let url = elem.attr("href").unwrap().to_string();
                    let value = if let Some(link_text) = inner_child.value().as_text() {
                        trim_first(index, link_text)
                    } else {
                        url.clone()
                    };
                    return Some(TypedText::Link {
                        url,
                        value
                    })
                }
                if let Some(text) = inner_child.value().as_text().map(|txt| trim_first(index, txt)) {
                    match elem.name() {
                        "code" => {
                            return Some(TypedText::Code(text))
                        },
                        "i" | "em" => {
                            return Some(TypedText::Italic(text))
                        },
                        "strong" => {
                            return Some(TypedText::Strong(text))
                        },
                        _tag_name => {
                            log::warn!("Unknown tag inside paragraph: {_tag_name}");
                        }
                    }
                }
            }
        }
        None
    }).collect()
}

fn get_element_text<'a>(element: &ElementRef<'a>) -> String {
    element
        .text()
        .map(|txt| txt.trim())
        .collect::<Vec<&str>>()
        .join(" ")
}

fn get_list_items<'a>(element: &ElementRef<'a>) -> Vec<ArticleContent> {
    element.children().filter_map(|child| {
        if child.value().is_element() {
            if let Some(li_child) = child.first_child() {
                if let Some(text) = li_child.value().as_text() {
                    Some(ArticleContent::Text(TypedText::Common(text.trim().to_string())))
                } else if li_child.value().is_element() {
                    let res = ArticleContent::Paragraph(extract_paragraph_content(&ElementRef::wrap(li_child).unwrap()));
                    Some(res)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }).collect()
}

fn parse_recursively<'a>(element: &ElementRef<'a>) -> Vec<ArticleContent> {
    match element.value().name() {
        "img" => {
            if let Some(img_src) = element.attr("src") {
                vec![ArticleContent::Image(img_src.to_string())]
            } else {
                vec![]
            }
        }
        "figure" => {
            if let Some(image_child) = element.first_child() {
                return parse_recursively(&ElementRef::wrap(image_child).unwrap())
            }
            vec![]
        }
        "p" => vec![ArticleContent::Paragraph(extract_paragraph_content(element))],
        "h2" => vec![ArticleContent::Header(2, get_element_text(element))],
        "h3" => vec![ArticleContent::Header(3, get_element_text(element))],
        "h4" => vec![ArticleContent::Header(4, get_element_text(element))],
        "pre" => {
            if let Some(f_child) = element.first_child() {
                if let Some(text) = f_child.value().as_text() {
                    return vec![ArticleContent::Code{
                        lang: element.attr("class").unwrap_or("").to_string(),
                        content: text.to_string()
                    }]
                }
                return parse_recursively(&ElementRef::wrap(f_child).unwrap())
            }
            vec![ArticleContent::Code{
                lang: element.attr("class").unwrap_or("").to_string(),
                content: get_element_text(element)
            }]
        }
        "code" => {
            vec![ArticleContent::Code{
                lang: element.attr("class").unwrap_or("").to_string(),
                content: get_element_text(element)
            }]
        },
        "blockquote" => {
            vec![ArticleContent::Blockquote(get_element_text(element))]
        },
        "ul" => {
            vec![ArticleContent::UnorderedList(get_list_items(element))]
        },
        "ol" => {
            vec![ArticleContent::OrderedList(get_list_items(element))]
        },
        "a" => {
            let url = element.attr("href").unwrap_or("").to_string();
            let link_text = get_element_text(element);
            let value = if !link_text.is_empty() {
                link_text
            } else {
                url.clone()
            };
            vec![ArticleContent::Paragraph(vec![TypedText::Link {url, value}])]
        },
        "i" => {
            vec![ArticleContent::Paragraph(vec![TypedText::Italic(get_element_text(element))])]
        }
        "div" => {
            element.children().flat_map(|child| {
                if let Some(inner_elem) = ElementRef::wrap(child) {
                    parse_recursively(&inner_elem)
                } else if let Some(text) = child.value().as_text() {
                    let text = text.trim().to_string();
                    if text.is_empty() {
                        vec![]
                    } else {
                        vec![ArticleContent::Text(TypedText::Common(text))]
                    }
                } else {
                    vec![]
                }
            }).collect()
        }
        "br" => {
            vec![ArticleContent::BR]
        }
        _tag @ _ => {
            log::warn!("[!] Unsupported tag: {} with content: {}, {:?}", _tag, element.html(), element.attr("class"));
            vec![]
        }
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
