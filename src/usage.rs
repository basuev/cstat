use std::fs;

use serde::Deserialize;

use crate::types::{State, UsageCache};

pub struct UsageInfo {
    pub usage_5h: Option<f64>,
    pub usage_7d: Option<f64>,
    pub reset_5h: Option<i64>,
}

const CACHE_TTL_SUCCESS: i64 = 60;
const CACHE_TTL_FAILURE: i64 = 15;
const USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";

#[derive(Debug, Deserialize)]
struct Credentials {
    #[serde(rename = "claudeAiOauth")]
    claude_ai_oauth: Option<OAuthBlock>,
}

#[derive(Debug, Deserialize)]
struct OAuthBlock {
    #[serde(rename = "accessToken")]
    access_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UsageResponse {
    #[serde(rename = "fiveHourUsage")]
    five_hour_usage: Option<UsageWindow>,
    #[serde(rename = "sevenDayUsage")]
    seven_day_usage: Option<UsageWindow>,
}

#[derive(Debug, Deserialize)]
struct UsageWindow {
    percentage: Option<f64>,
    #[serde(rename = "resetsAt")]
    resets_at: Option<String>,
}

fn read_token() -> Option<String> {
    let home = dirs::home_dir()?;
    let path = home.join(".claude").join(".credentials.json");
    let data = fs::read_to_string(&path).ok()?;
    let creds: Credentials = serde_json::from_str(&data).ok()?;
    creds.claude_ai_oauth?.access_token
}

fn parse_reset_time(iso: &str) -> Option<i64> {
    let dt = chrono::DateTime::parse_from_rfc3339(iso).ok()?;
    let now = chrono::Utc::now().timestamp();
    let reset = dt.timestamp();
    let remaining = reset - now;
    if remaining > 0 { Some(remaining) } else { None }
}

pub fn parse_usage_response(body: &str) -> Option<UsageInfo> {
    let resp: UsageResponse = serde_json::from_str(body).ok()?;
    let usage_5h = resp.five_hour_usage.as_ref().and_then(|w| w.percentage);
    let usage_7d = resp.seven_day_usage.as_ref().and_then(|w| w.percentage);
    let reset_5h = resp
        .five_hour_usage
        .as_ref()
        .and_then(|w| w.resets_at.as_deref())
        .and_then(parse_reset_time);
    if usage_5h.is_none() && usage_7d.is_none() {
        return None;
    }
    Some(UsageInfo {
        usage_5h,
        usage_7d,
        reset_5h,
    })
}

fn fetch_from_api(token: &str) -> Option<UsageInfo> {
    let resp = ureq::get(USAGE_URL)
        .set("Authorization", &format!("Bearer {token}"))
        .call()
        .ok()?;
    if resp.status() != 200 {
        return None;
    }
    let body = resp.into_string().ok()?;
    parse_usage_response(&body)
}

fn cache_valid(cache: &UsageCache, now: i64) -> bool {
    let ttl = if cache.success {
        CACHE_TTL_SUCCESS
    } else {
        CACHE_TTL_FAILURE
    };
    now - cache.fetched_at < ttl
}

pub fn fetch_usage(state: &mut State) -> Option<UsageInfo> {
    let now = chrono::Utc::now().timestamp();

    if let Some(cache) = state.usage_cache.as_ref().filter(|c| cache_valid(c, now)) {
        if cache.success {
            return Some(UsageInfo {
                usage_5h: cache.usage_5h,
                usage_7d: cache.usage_7d,
                reset_5h: cache.reset_5h,
            });
        }
        return None;
    }

    let token = read_token()?;
    match fetch_from_api(&token) {
        Some(info) => {
            state.usage_cache = Some(UsageCache {
                fetched_at: now,
                success: true,
                usage_5h: info.usage_5h,
                usage_7d: info.usage_7d,
                reset_5h: info.reset_5h,
            });
            Some(info)
        }
        None => {
            state.usage_cache = Some(UsageCache {
                fetched_at: now,
                success: false,
                usage_5h: None,
                usage_7d: None,
                reset_5h: None,
            });
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_response() {
        let body = r#"{
            "fiveHourUsage": {"percentage": 25.0, "resetsAt": "2099-01-01T00:00:00Z"},
            "sevenDayUsage": {"percentage": 60.0, "resetsAt": "2099-01-07T00:00:00Z"}
        }"#;
        let info = parse_usage_response(body).unwrap();
        assert!((info.usage_5h.unwrap() - 25.0).abs() < 0.01);
        assert!((info.usage_7d.unwrap() - 60.0).abs() < 0.01);
        assert!(info.reset_5h.unwrap() > 0);
    }

    #[test]
    fn parse_missing_5h() {
        let body = r#"{
            "sevenDayUsage": {"percentage": 40.0}
        }"#;
        let info = parse_usage_response(body).unwrap();
        assert!(info.usage_5h.is_none());
        assert!((info.usage_7d.unwrap() - 40.0).abs() < 0.01);
    }

    #[test]
    fn parse_missing_7d() {
        let body = r#"{
            "fiveHourUsage": {"percentage": 10.0}
        }"#;
        let info = parse_usage_response(body).unwrap();
        assert!((info.usage_5h.unwrap() - 10.0).abs() < 0.01);
        assert!(info.usage_7d.is_none());
    }

    #[test]
    fn parse_empty_response() {
        let body = r#"{}"#;
        assert!(parse_usage_response(body).is_none());
    }

    #[test]
    fn parse_invalid_json() {
        assert!(parse_usage_response("not json").is_none());
    }

    #[test]
    fn parse_reset_past() {
        let body = r#"{
            "fiveHourUsage": {"percentage": 50.0, "resetsAt": "2000-01-01T00:00:00Z"},
            "sevenDayUsage": {"percentage": 30.0}
        }"#;
        let info = parse_usage_response(body).unwrap();
        assert!(info.reset_5h.is_none());
    }

    #[test]
    fn cache_success_valid_within_ttl() {
        let now = chrono::Utc::now().timestamp();
        let cache = UsageCache {
            fetched_at: now - 30,
            success: true,
            usage_5h: Some(25.0),
            usage_7d: Some(60.0),
            reset_5h: Some(3600),
        };
        assert!(cache_valid(&cache, now));
    }

    #[test]
    fn cache_success_expired_after_ttl() {
        let now = chrono::Utc::now().timestamp();
        let cache = UsageCache {
            fetched_at: now - 61,
            success: true,
            usage_5h: Some(25.0),
            usage_7d: Some(60.0),
            reset_5h: None,
        };
        assert!(!cache_valid(&cache, now));
    }

    #[test]
    fn cache_failure_valid_within_ttl() {
        let now = chrono::Utc::now().timestamp();
        let cache = UsageCache {
            fetched_at: now - 10,
            success: false,
            usage_5h: None,
            usage_7d: None,
            reset_5h: None,
        };
        assert!(cache_valid(&cache, now));
    }

    #[test]
    fn cache_failure_expired_after_ttl() {
        let now = chrono::Utc::now().timestamp();
        let cache = UsageCache {
            fetched_at: now - 16,
            success: false,
            usage_5h: None,
            usage_7d: None,
            reset_5h: None,
        };
        assert!(!cache_valid(&cache, now));
    }

    #[test]
    fn fetch_usage_returns_cached_success() {
        let now = chrono::Utc::now().timestamp();
        let mut state = State {
            usage_cache: Some(UsageCache {
                fetched_at: now - 5,
                success: true,
                usage_5h: Some(25.0),
                usage_7d: Some(60.0),
                reset_5h: Some(3600),
            }),
            ..Default::default()
        };
        let info = fetch_usage(&mut state).unwrap();
        assert!((info.usage_5h.unwrap() - 25.0).abs() < 0.01);
        assert!((info.usage_7d.unwrap() - 60.0).abs() < 0.01);
    }

    #[test]
    fn fetch_usage_returns_none_for_cached_failure() {
        let now = chrono::Utc::now().timestamp();
        let mut state = State {
            usage_cache: Some(UsageCache {
                fetched_at: now - 5,
                success: false,
                usage_5h: None,
                usage_7d: None,
                reset_5h: None,
            }),
            ..Default::default()
        };
        assert!(fetch_usage(&mut state).is_none());
    }

    #[test]
    fn fetch_usage_no_credentials_returns_none() {
        let mut state = State::default();
        assert!(fetch_usage(&mut state).is_none());
    }

    #[test]
    fn read_token_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let claude_dir = dir.path().join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();
        let cred_path = claude_dir.join(".credentials.json");
        fs::write(
            &cred_path,
            r#"{"claudeAiOauth": {"accessToken": "test-token-123"}}"#,
        )
        .unwrap();

        let data = fs::read_to_string(&cred_path).unwrap();
        let creds: Credentials = serde_json::from_str(&data).unwrap();
        let token = creds.claude_ai_oauth.unwrap().access_token.unwrap();
        assert_eq!(token, "test-token-123");
    }

    #[test]
    fn read_token_missing_field() {
        let data = r#"{"someOtherField": true}"#;
        let creds: Credentials = serde_json::from_str(data).unwrap();
        assert!(creds.claude_ai_oauth.is_none());
    }
}
