use lazy_static::lazy_static;
use log::info;

const BUCKET_URL: &str = "https://storage.googleapis.com/image-resizer_europe-west1";

lazy_static! {
    static ref BUCKET_CLIENT: reqwest::Client = bucket_client();
}

pub async fn bucket_request(path: &str) -> Result<Vec<u8>, reqwest::Error> {
    let url = String::from(BUCKET_URL) + path;
    let resp = BUCKET_CLIENT.get(&url).send().await?;
    resp.bytes().await.map(|d| d.to_vec())
}

fn bucket_client() -> reqwest::Client {
    info!("Initializing bucket client.");
    reqwest::Client::builder()
        .https_only(true)
        .use_rustls_tls()
        .connection_verbose(true)
        .build()
        .unwrap()
}
