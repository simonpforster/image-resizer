use crate::logging::log_message::LogMessage;
use chrono::{DateTime, Local};
use log::Record;
use log4rs::encode::{Encode, Write};
use serde::Serialize;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct CloudEncoder(());

impl CloudEncoder {
    pub fn new() -> Self {
        Self::default()
    }
}

impl CloudEncoder {
    fn encode_inner(
        &self,
        w: &mut dyn Write,
        time: DateTime<Local>,
        record: &Record,
    ) -> anyhow::Result<()> {
        let message = LogMessage {
            time: Some(time.to_rfc3339()),
            severity: Some(String::from(record.level().as_str())),
            message: Some(record.args().to_string()),
            trace: None,
        };
        message.serialize(&mut serde_json::Serializer::new(&mut *w))?;
        w.write_all("\n".as_bytes())?;
        Ok(())
    }
}

impl Encode for CloudEncoder {
    fn encode(&self, w: &mut dyn Write, record: &Record) -> anyhow::Result<()> {
        self.encode_inner(w, Local::now(), record)
    }
}


