use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::habr_client::article::{ArticleContent, ArticleData};

#[derive(Serialize, Deserialize)]
struct SavedArticle {
    metadata: ArticleData,
    content: Vec<ArticleContent>,
    saved_at: String,
}

pub fn app_data_dir() -> PathBuf {
    let home_dir = std::env::home_dir().unwrap();
    home_dir.join(".local/share/com.lmaxyz/Haboost")
}

pub struct ArticleStorage;

impl ArticleStorage {
    fn base_path() -> PathBuf {
        app_data_dir().join("saved_articles")
    }

    fn article_path(article_id: &str) -> PathBuf {
        Self::base_path().join(article_id)
    }

    fn images_path(article_id: &str) -> PathBuf {
        Self::article_path(article_id).join("images")
    }

    pub fn is_article_saved(article_id: &str) -> bool {
        Self::article_path(article_id).join("article.json").exists()
    }

    pub fn list_saved_articles() -> Vec<ArticleData> {
        let base = Self::base_path();
        if !base.exists() {
            return Vec::new();
        }

        let mut articles = Vec::new();
        if let Ok(entries) = fs::read_dir(base) {
            for entry in entries.flatten() {
                let path = entry.path().join("article.json");
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(saved) = serde_json::from_str::<SavedArticle>(&content) {
                        articles.push(saved.metadata);
                    }
                }
            }
        }
        articles
    }

    pub fn load_article(article_id: &str) -> Option<(ArticleData, Vec<ArticleContent>)> {
        let path = Self::article_path(article_id).join("article.json");
        if !path.exists() {
            return None;
        }

        let content = fs::read_to_string(&path).ok()?;
        let saved: SavedArticle = serde_json::from_str(&content).ok()?;
        Some((saved.metadata, saved.content))
    }

    pub async fn save_article(data: &ArticleData, content: &[ArticleContent]) -> Result<(), String> {
        let article_path = Self::article_path(&data.id);
        let images_path = Self::images_path(&data.id);

        fs::create_dir_all(&images_path).map_err(|e| e.to_string())?;

        let mut url_map = HashMap::new();

        let image_urls = Self::collect_image_urls(content);
        for url in &image_urls {
            if let Some(local_path) = Self::download_image(url, &images_path).await {
                url_map.insert(url.clone(), local_path);
            }
        }

        if !data.image_url.is_empty() {
            if let Some(local_path) = Self::download_image(&data.image_url, &images_path).await {
                url_map.insert(data.image_url.clone(), local_path);
            }
        }

        let mut saved_content = content.to_vec();
        Self::replace_image_urls(&mut saved_content, &url_map);

        let mut saved_metadata = data.clone();
        if let Some(local_path) = url_map.get(&data.image_url) {
            saved_metadata.image_url = local_path.clone();
        }

        let saved_article = SavedArticle {
            metadata: saved_metadata,
            content: saved_content,
            saved_at: chrono::Local::now().format("%d.%m.%Y %H:%M").to_string(),
        };

        let json = serde_json::to_string_pretty(&saved_article).map_err(|e| e.to_string())?;
        fs::write(article_path.join("article.json"), json).map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn delete_article(article_id: &str) -> Result<(), String> {
        let path = Self::article_path(article_id);
        if path.exists() {
            fs::remove_dir_all(&path).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn collect_image_urls(content: &[ArticleContent]) -> Vec<String> {
        let mut urls = Vec::new();
        for item in content {
            match item {
                ArticleContent::Image(url) => urls.push(url.clone()),
                ArticleContent::UnorderedList(list) | ArticleContent::OrderedList(list) => {
                    urls.extend(Self::collect_image_urls(list));
                }
                _ => {}
            }
        }
        urls
    }

    fn replace_image_urls(content: &mut [ArticleContent], url_map: &HashMap<String, String>) {
        for item in content {
            match item {
                ArticleContent::Image(url) => {
                    if let Some(local) = url_map.get(url) {
                        *url = local.clone();
                    }
                }
                ArticleContent::UnorderedList(list) | ArticleContent::OrderedList(list) => {
                    Self::replace_image_urls(list, url_map);
                }
                _ => {}
            }
        }
    }

    async fn download_image(url: &str, images_dir: &std::path::Path) -> Option<String> {
        if url.starts_with("file://") {
            return Some(url.to_string());
        }

        let response = match reqwest::get(url).await {
            Ok(r) => r,
            Err(e) => {
                log::warn!("Failed to download image {}: {}", url, e);
                return None;
            }
        };

        let bytes = match response.bytes().await {
            Ok(b) => b,
            Err(e) => {
                log::warn!("Failed to read image bytes {}: {}", url, e);
                return None;
            }
        };

        let ext = Self::guess_extension(url, &bytes);
        let hash = Self::hash_url(url);
        let filename = format!("{}{}", hash, ext);
        let filepath = images_dir.join(&filename);

        if let Err(e) = fs::write(&filepath, &bytes) {
            log::warn!("Failed to write image {}: {}", filename, e);
            return None;
        }

        Some(format!("file://{}", filepath.to_string_lossy()))
    }

    fn hash_url(url: &str) -> String {
        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    fn guess_extension(url: &str, bytes: &[u8]) -> &'static str {
        if bytes.starts_with(b"\x89PNG") {
            return ".png";
        }
        if bytes.starts_with(b"\xFF\xD8\xFF") {
            return ".jpg";
        }
        if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
            return ".gif";
        }
        if bytes.starts_with(b"RIFF") && bytes.len() > 8 && &bytes[8..12] == b"WEBP" {
            return ".webp";
        }

        if let Some(ext) = url.split('.').next_back() {
            match ext.to_lowercase().as_str() {
                "png" => return ".png",
                "jpg" | "jpeg" => return ".jpg",
                "gif" => return ".gif",
                "webp" => return ".webp",
                "bmp" => return ".bmp",
                _ => {}
            }
        }

        ".bin"
    }
}
