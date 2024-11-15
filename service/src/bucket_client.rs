use lazy_static::lazy_static;
use log::info;
use tokio::time::Instant;

const BUCKET_URL: &str = "https://storage.googleapis.com/image-resizer_europe-west1";

lazy_static! {
    static ref CLIENT: reqwest::Client = reqwest::Client::new();
}

pub async fn bucket_request(path: &str) -> Result<Vec<u8>, reqwest::Error> {
    let http_request_timer = Instant::now();
    let url = String::from(BUCKET_URL) + path;
    let resp = CLIENT.get(&url).send().await?;
    let status = resp.status();
    let headers  = resp.headers().clone();
    let version = resp.version();
    let bytes = resp.bytes().await.map(|d| { d.to_vec() });
    info!("Code {}:{:?} after {} for {}", status, version, http_request_timer.elapsed().as_millis(), path);
    info!("headers {:#?}", headers);
    bytes
}
