use hyper::Version;
use lazy_static::lazy_static;
use log::info;
use tokio::time::Instant;

const BUCKET_URL: &str = "https://storage.googleapis.com/image-resizer_europe-west1";

lazy_static! {
    static ref CLIENT: reqwest::Client = bucket_client();
}

pub async fn bucket_request(path: &str) -> Result<Vec<u8>, reqwest::Error> {
    let http_request_timer = Instant::now();
    let url = String::from(BUCKET_URL) + path;
    let resp = CLIENT.get(&url).version(Version::HTTP_3).send().await?;
    let status = resp.status();
    let headers  = resp.headers().clone();
    let version = resp.version();
    let bytes = resp.bytes().await.map(|d| { d.to_vec() });
    info!("Code {}:{:?} after {} for {}", status, version, http_request_timer.elapsed().as_millis(), path);
    info!("headers {:#?}", headers);
    bytes
}

fn bucket_client() -> reqwest::Client {
    reqwest::Client::builder()
        .https_only(true)
        .http3_prior_knowledge()
        .use_rustls_tls()
        .connection_verbose(true)
        .build().unwrap()
}