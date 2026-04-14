//! Builds the `action` envelope the host receives when we emit
//! `Response::Action`. Shape is fixed by the skill-author contract in
//! `docs/action-responses.md`:
//!
//! ```json
//! {
//!   "action": "timer",
//!   "speak": "...",
//!   "events": [ { "kind": "...", ... }, ... ],
//!   "timers": [ <full authoritative snapshot> ]
//! }
//! ```

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use serde::Serialize;

use crate::state::{State, Timer};

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Event {
    Create {
        id: String,
        name: Option<String>,
        duration_ms: u64,
        end_ts_ms: i64,
        created_ts_ms: i64,
    },
    Cancel {
        id: String,
    },
    CancelAll,
    /// Skill acknowledges the utterance but made no change (query / list /
    /// disambiguation). Frontend reads `speak` and reconciles `timers`.
    Ack,
}

#[derive(Debug, Serialize)]
pub struct Envelope<'a> {
    pub action: &'static str,
    pub speak: String,
    pub events: Vec<Event>,
    pub timers: &'a [Timer],
}

impl<'a> Envelope<'a> {
    pub fn new(speak: impl Into<String>, events: Vec<Event>, state: &'a State) -> Self {
        Envelope {
            action: "timer",
            speak: speak.into(),
            events,
            timers: &state.timers,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }
}
