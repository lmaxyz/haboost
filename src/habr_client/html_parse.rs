use scraper::{ElementRef, Html};

use super::{ArticleContent};


pub fn extract_text_from_html(input: &str) -> String {
    let html = Html::parse_fragment(&input);
    get_element_text(&html.root_element())
}

pub async fn extract_content_from_html(text: String) -> Vec<ArticleContent> {
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
