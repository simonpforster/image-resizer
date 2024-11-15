#[derive(serde::Serialize)]
pub struct LogMessage {
    pub severity: Option<String>,
    pub message: Option<String>,
    pub time: Option<String>,
}