use std::fmt::{Display, Formatter};
use std::time::{Duration, Instant};
use log4rs::config::InitError::Deserializing;

#[derive(Debug, Clone)]
pub struct Timing {
    pub name: String,
    pub duration: String,
    pub description: Option<String>,
}

impl Display for Timing {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.description {
            Some(desc) => write!(f, "{};desc=\"{}\";dur={}", self.name, desc, self.duration),
            None => write!(f, "{};dur={}", self.name, self.duration),
        }
    }
}

impl Timing {
    pub fn new(name: &str, duration: Duration, descrition: Option<String>) -> Timing {
        let dur = duration.as_millis().to_string();
        Timing { name: name.to_string(), duration: dur.to_string(), description: descrition }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timing_display_description() {
        let name = String::from("db");
        let dur = String::from("65.3");
        let desc = String::from("Do a thing on the db");
        let timing = Timing {
            name: name.clone(),
            duration: dur.clone(),
            description: Some(desc.clone()),
        };
        assert_eq!(format!("{timing}"), format!("{name};desc=\"{desc}\";dur={dur}"))
    }

    #[test]
    fn timing_display_no_description() {
        let name = String::from("db");
        let dur = String::from("65.3");
        let timing = Timing {
            name: name.clone(),
            duration: dur.clone(),
            description: None,
        };
        assert_eq!(format!("{timing}"), format!("{name};dur={dur}"))
    }
}

