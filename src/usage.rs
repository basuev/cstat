use crate::types::State;

pub struct UsageInfo {
    pub usage_5h: Option<f64>,
    pub usage_7d: Option<f64>,
    pub reset_5h: Option<i64>,
}

pub fn fetch_usage(_state: &mut State) -> Option<UsageInfo> {
    None
}
