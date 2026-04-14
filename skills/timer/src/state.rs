//! Timer state persisted in the skill's `storage_kv` file.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

/// Persisted JSON shape. Bump `v` when the layout changes in a way that
/// can't be read back compatibly — a fresh install reads `v=1` and any old
/// file with a different `v` is discarded.
pub const STATE_SCHEMA_VERSION: u32 = 1;
pub const STATE_KEY: &str = "state";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Timer {
    pub id: String,
    pub name: Option<String>,
    pub end_ts_ms: i64,
    pub created_ts_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    #[serde(default = "default_version")]
    pub v: u32,
    #[serde(default)]
    pub timers: Vec<Timer>,
}

impl Default for State {
    fn default() -> Self {
        State { v: STATE_SCHEMA_VERSION, timers: Vec::new() }
    }
}

fn default_version() -> u32 {
    STATE_SCHEMA_VERSION
}

impl State {
    pub fn load(raw: &str) -> State {
        match serde_json::from_str::<State>(raw) {
            Ok(s) if s.v == STATE_SCHEMA_VERSION => s,
            _ => State::default(),
        }
    }

    /// Remove every timer whose end time is at or before `now`. Returns the
    /// ids that were pruned, which callers emit as `cancel` events so the
    /// frontend can dismiss any lingering card/notification.
    pub fn prune_expired(&mut self, now: i64) -> Vec<String> {
        let mut pruned = Vec::new();
        self.timers.retain(|t| {
            if t.end_ts_ms <= now {
                pruned.push(t.id.clone());
                false
            } else {
                true
            }
        });
        pruned
    }

    pub fn find_by_name(&self, name: &str) -> Option<&Timer> {
        self.timers.iter().find(|t| match &t.name {
            Some(n) => n.eq_ignore_ascii_case(name),
            None => false,
        })
    }

    /// Remove one timer by name. Returns its id if found.
    pub fn remove_by_name(&mut self, name: &str) -> Option<String> {
        if let Some(pos) = self.timers.iter().position(|t| match &t.name {
            Some(n) => n.eq_ignore_ascii_case(name),
            None => false,
        }) {
            Some(self.timers.remove(pos).id)
        } else {
            None
        }
    }

    /// Remove the single anonymous timer (name = None). Returns its id if
    /// exactly one exists.
    pub fn remove_only_anonymous(&mut self) -> Option<String> {
        let anon_positions: Vec<usize> = self
            .timers
            .iter()
            .enumerate()
            .filter_map(|(i, t)| if t.name.is_none() { Some(i) } else { None })
            .collect();
        if anon_positions.len() == 1 {
            Some(self.timers.remove(anon_positions[0]).id)
        } else {
            None
        }
    }

    pub fn serialise(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{\"v\":1,\"timers\":[]}".to_string())
    }
}
