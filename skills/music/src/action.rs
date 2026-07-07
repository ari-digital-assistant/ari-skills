extern crate alloc;
use alloc::string::{String, ToString};

pub fn play_action_json(query: &str, service: &str) -> String {
    serde_json::json!({
        "v": 1,
        "media": { "action": "play", "query": query, "service": service }
    })
    .to_string()
}

use crate::transport::Transport;

pub fn transport_action_json(t: &Transport) -> String {
    let media = match t {
        Transport::Pause => serde_json::json!({ "action": "pause" }),
        Transport::Resume => serde_json::json!({ "action": "resume" }),
        Transport::Next => serde_json::json!({ "action": "next" }),
        Transport::Previous => serde_json::json!({ "action": "previous" }),
        Transport::Stop => serde_json::json!({ "action": "stop" }),
        Transport::VolumeUp => serde_json::json!({ "action": "volume", "direction": "up" }),
        Transport::VolumeDown => serde_json::json!({ "action": "volume", "direction": "down" }),
        Transport::VolumeSet(n) => serde_json::json!({ "action": "volume", "level": n }),
        Transport::Mute => serde_json::json!({ "action": "volume", "mute": true }),
        Transport::Unmute => serde_json::json!({ "action": "volume", "mute": false }),
    };
    serde_json::json!({ "v": 1, "media": media }).to_string()
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

    use crate::transport::Transport;

    #[test]
    fn pause_action_shape() {
        let v: serde_json::Value =
            serde_json::from_str(&transport_action_json(&Transport::Pause)).unwrap();
        assert_eq!(v["v"], 1);
        assert_eq!(v["media"]["action"], "pause");
        assert!(v["media"].get("query").is_none());
    }

    #[test]
    fn next_previous_resume_stop_shapes() {
        let a = |t| serde_json::from_str::<serde_json::Value>(&transport_action_json(t)).unwrap();
        assert_eq!(a(&Transport::Next)["media"]["action"], "next");
        assert_eq!(a(&Transport::Previous)["media"]["action"], "previous");
        assert_eq!(a(&Transport::Resume)["media"]["action"], "resume");
        assert_eq!(a(&Transport::Stop)["media"]["action"], "stop");
    }

    #[test]
    fn volume_shapes() {
        let a = |t| serde_json::from_str::<serde_json::Value>(&transport_action_json(t)).unwrap();
        let up = a(&Transport::VolumeUp);
        assert_eq!(up["media"]["action"], "volume");
        assert_eq!(up["media"]["direction"], "up");
        let down = a(&Transport::VolumeDown);
        assert_eq!(down["media"]["direction"], "down");
        let set = a(&Transport::VolumeSet(50));
        assert_eq!(set["media"]["action"], "volume");
        assert_eq!(set["media"]["level"], 50);
        let mute = a(&Transport::Mute);
        assert_eq!(mute["media"]["mute"], true);
        let unmute = a(&Transport::Unmute);
        assert_eq!(unmute["media"]["mute"], false);
    }
}
