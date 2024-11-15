use lazy_static::lazy_static;
use log::info;

const BUCKET_URL: &str = "https://storage.googleapis.com/image-resizer_europe-west1";

lazy_static! {
    static ref CLIENT: reqwest::Client = bucket_client();
}

pub async fn bucket_request(path: &str) -> Result<Vec<u8>, reqwest::Error> {
    let url = String::from(BUCKET_URL) + path;
    let resp = CLIENT.get(&url).send().await?;
    let bytes = resp.bytes().await.map(|d| { d.to_vec() });
    bytes
}

fn bucket_client() -> reqwest::Client {
    info!("init client");
    reqwest::Client::builder()
        .https_only(true)
        .use_rustls_tls()
        .connection_verbose(true)
        .build().unwrap()
}