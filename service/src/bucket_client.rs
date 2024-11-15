use image::EncodableLayout;

const BUCKET_URL: &str = "https://storage.googleapis.com/image-resizer_europe-west1";


pub async fn bucket_request(path: &str) -> Option<&[u8]> {
    let url = String::from(BUCKET_URL) + path;
    let res = reqwest::get(&url).await;
    let resp = res.unwrap();
    let bytes = resp.bytes().await.map(|v| v.as_bytes()).ok();
    bytes
}