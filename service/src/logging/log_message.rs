#[derive(serde::Serialize)]
pub struct LogMessage {
    pub severity: Option<String>,
    pub message: Option<String>,
    pub time: Option<String>,
    #[serde(rename = "logging.googleapis.com/trace")]
    pub trace: Option<String>,
}