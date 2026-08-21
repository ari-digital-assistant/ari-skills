extern crate alloc;
use alloc::string::String;
use ari_skill_sdk::presentation as p;

pub fn play_action_json(query: &str, service: &str) -> String {
    p::Envelope::new().media(p::Media::play(query).service(service)).to_json()
}

use crate::transport::Transport;

pub fn transport_action_json(t: &Transport) -> String {
    let media = match t {
        Transport::Pause => p::Media::pause(),
        Transport::Resume => p::Media::resume(),
        Transport::Next => p::Media::next(),
        Transport::Previous => p::Media::previous(),
        Transport::Stop => p::Media::stop(),
        Transport::VolumeUp => p::Media::volume_up(),
        Transport::VolumeDown => p::Media::volume_down(),
        Transport::VolumeSet(n) => p::Media::volume(*n),
        Transport::Mute => p::Media::mute(),
        Transport::Unmute => p::Media::unmute(),
    };
    p::Envelope::new().media(media).to_json()
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

    /// The builder replaced a hand-written `json!` here, so pin the wire
    /// format outright rather than field by field — a reordered or renamed
    /// key would still satisfy the assertions above.
    #[test]
    fn play_action_bytes_are_unchanged_by_the_builder() {
        assert_eq!(
            play_action_json("hotel california", "spotify"),
            r#"{"v":1,"media":{"action":"play","query":"hotel california","service":"spotify"}}"#,
        );
        assert_eq!(
            transport_action_json(&Transport::VolumeSet(50)),
            r#"{"v":1,"media":{"action":"volume","level":50}}"#,
        );
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
