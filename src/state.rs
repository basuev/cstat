use crate::types::State;

pub fn load_state(_transcript_path: Option<&str>) -> State {
    State::default()
}

pub fn save_state(_state: &State, _transcript_path: Option<&str>) {}
