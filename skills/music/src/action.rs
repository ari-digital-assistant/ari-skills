extern crate alloc;
use alloc::string::{String, ToString};

pub fn play_action_json(query: &str, service: &str) -> String {
    serde_json::json!({
        "v": 1,
        "media": { "action": "play", "query": query, "service": service }
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn play_action_shape() {
        let j = play_action_json("hotel california", "spotify");
        let v: serde_json::Value = serde_json::from_str(&j).unwrap();
        assert_eq!(v["v"], 1);
        assert_eq!(v["media"]["action"], "play");
        assert_eq!(v["media"]["query"], "hotel california");
        assert_eq!(v["media"]["service"], "spotify");
        assert!(v.get("speak").is_none());
    }
}
