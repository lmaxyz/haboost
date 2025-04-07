use image::DynamicImage;

pub async fn donwload_img(url: &str) -> Option<DynamicImage> {
    if url.is_empty() {
        return None;
    }

    if let Ok(resp) = reqwest::get(url).await {
        if let Ok(bytes) = resp.bytes().await {
            return image::load_from_memory(bytes.as_ref()).ok();
        }
    }

    return None;
}
