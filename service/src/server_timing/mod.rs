use std::fmt::{Display, Formatter};
use crate::server_timing::timing::Timing;

pub mod timing;

pub struct ServerTiming {
    timings: Vec<Timing>,
}

impl Display for ServerTiming {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let display =
            self.timings.iter()
                .map(|tim| { tim.to_string() })
                .reduce(|acc, val| {
                    format!("{acc}, {val}")
                }).unwrap();

        write!(f, "{display}")
    }
}

impl ServerTiming {
    pub fn new(timings: Vec<Timing>) -> ServerTiming {
        ServerTiming { timings }
    }
}

mod tests {
    use super::*;

    #[test]
    fn server_timing_display() {
        let timing1 = Timing {
            name: String::from("db"),
            duration: String::from("56.6"),
            description: None,
        };
        let timing2 = Timing {
            name: String::from("req"),
            duration: String::from("32.3"),
            description: None,
        };
        let expected_string = format!("{timing1}, {timing2}");

        let server_timing = ServerTiming::new([timing1, timing2].to_vec());
        let formatted_string = format!("{server_timing}");

        assert_eq!(expected_string, formatted_string)
    }
}