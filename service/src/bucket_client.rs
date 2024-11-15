use image::EncodableLayout;

const BUCKET_URL: &str = "https://storage.googleapis.com/image-resizer_europe-west1";


pub async fn bucket_request(path: &str) -> Option<Vec<u8>> {
    let url = String::from(BUCKET_URL) + path;
    let res = reqwest::get(&url).await;
    let resp = res.unwrap();
    let bytes = resp.bytes().await.ok().map(|d| {d.as_bytes().to_vec()});
    bytes
}