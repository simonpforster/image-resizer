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
    use std::time::Duration;
    use super::*;

    #[test]
    fn server_timing_display() {
        let timing1 = Timing::new("db", Duration::from_millis(56), None);
        let timing2 = Timing::new("ser", Duration::from_millis(32), None);
        let expected_string = format!("{timing1}, {timing2}");

        let server_timing = ServerTiming::new([timing1, timing2].to_vec());
        let formatted_string = format!("{server_timing}");

        assert_eq!(expected_string, formatted_string)
    }
}