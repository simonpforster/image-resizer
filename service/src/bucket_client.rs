use image::EncodableLayout;
use log::info;
use tokio::time::Instant;

const BUCKET_URL: &str = "https://storage.googleapis.com/image-resizer_europe-west1";


pub async fn bucket_request(path: &str) -> Result<Vec<u8>, reqwest::Error> {
    let http_request_timer = Instant::now();
    let url = String::from(BUCKET_URL) + path;
    let resp = reqwest::get(&url).await?;
    let bytes = resp.bytes().await.map(|d| {d.as_bytes().to_vec()});
    info!("Code {} after {} for {}", resp.status(), http_request_timer.elapsed().as_millis(), path);
    info!("headers", resp.headers());
    bytes
}