use crate::types::StdinData;
use std::io::Read;

pub fn read_stdin() -> StdinData {
    let mut buf = String::new();
    if std::io::stdin().read_to_string(&mut buf).is_err() || buf.trim().is_empty() {
        return StdinData::default();
    }
    serde_json::from_str(&buf).unwrap_or_default()
}
