use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use crate::types::State;

const STATE_VERSION: u32 = 2;

fn state_path(transcript_path: &str) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    transcript_path.hash(&mut hasher);
    let hash = hasher.finish();
    PathBuf::from(format!("/tmp/cstat-{hash:x}.bin"))
}

pub fn load_state(transcript_path: Option<&str>) -> State {
    let Some(tp) = transcript_path else {
        return State::default();
    };

    let path = state_path(tp);
    let Ok(data) = fs::read(&path) else {
        return State::default();
    };

    match bincode::deserialize::<State>(&data) {
        Ok(s) if s.version == STATE_VERSION => s,
        _ => State::default(),
    }
}

pub fn save_state(state: &mut State, transcript_path: Option<&str>) {
    let Some(tp) = transcript_path else {
        return;
    };

    let path = state_path(tp);
    state.version = STATE_VERSION;

    if let Ok(data) = bincode::serialize(&state) {
        let _ = fs::write(&path, data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_path_deterministic() {
        let a = state_path("/some/path.jsonl");
        let b = state_path("/some/path.jsonl");
        assert_eq!(a, b);
    }

    #[test]
    fn state_path_differs_for_different_inputs() {
        let a = state_path("/a.jsonl");
        let b = state_path("/b.jsonl");
        assert_ne!(a, b);
    }

    #[test]
    fn missing_transcript_path_returns_default() {
        let s = load_state(None);
        assert_eq!(s.version, 0);
        assert_eq!(s.byte_offset, 0);
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let tp = dir.path().join("test.jsonl");
        let tp_str = tp.to_str().unwrap();

        let mut state = State::default();
        state.byte_offset = 42;
        state.inode = 123;
        save_state(&mut state, Some(tp_str));

        let loaded = load_state(Some(tp_str));
        assert_eq!(loaded.version, STATE_VERSION);
        assert_eq!(loaded.byte_offset, 42);
        assert_eq!(loaded.inode, 123);
    }

    #[test]
    fn incompatible_version_discarded() {
        let dir = tempfile::tempdir().unwrap();
        let tp = dir.path().join("test.jsonl");
        let tp_str = tp.to_str().unwrap();

        let mut state = State::default();
        state.version = 999;
        state.byte_offset = 100;
        let data = bincode::serialize(&state).unwrap();
        let path = state_path(tp_str);
        fs::write(&path, data).unwrap();

        let loaded = load_state(Some(tp_str));
        assert_eq!(loaded.version, 0);
        assert_eq!(loaded.byte_offset, 0);
    }

    #[test]
    fn corrupt_data_returns_default() {
        let dir = tempfile::tempdir().unwrap();
        let tp = dir.path().join("test.jsonl");
        let tp_str = tp.to_str().unwrap();

        let path = state_path(tp_str);
        fs::write(&path, b"garbage").unwrap();

        let loaded = load_state(Some(tp_str));
        assert_eq!(loaded.version, 0);
    }
}
