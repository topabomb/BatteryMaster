pub mod battery_status;
pub use battery_status::*;
#[cfg(test)]
mod tests {
    use crate::battery_status::*;
    use json::*;
    use std::str::FromStr;
    #[test]
    fn serialize() {
        let status = Status::default();
        let text = serde_json::to_string(&status);
        let parsed = parse(text.unwrap().as_str()).unwrap();
        assert_eq!(
            status.state,
            State(ExternalBatteryState::from_str(parsed["state"].as_str().unwrap()).unwrap())
        );
        let timestamp = parsed["timestamp"].clone();
        assert_eq!(status.timestamp, timestamp.as_i64().unwrap());
    }
    #[test]
    fn deserialize() {
        let status = Status::default();
        let text = serde_json::to_string(&status);
        let status1 = serde_json::from_str::<Status>(text.unwrap().as_str()).unwrap();
        assert_eq!(status, status1);
        assert_eq!(status.timestamp, status1.timestamp);
    }
}
