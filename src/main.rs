mod config;
mod git;
mod render;
mod state;
mod stdin;
mod transcript;
mod types;

use types::{CachedRateLimits, UsageInfo};

fn reset_secs(resets_at: Option<i64>, now: i64) -> Option<i64> {
    resets_at.map(|t| t - now).filter(|&r| r > 0)
}

fn usage_from_stdin(data: &types::StdinData) -> Option<(UsageInfo, CachedRateLimits)> {
    let rl = data.rate_limits.as_ref()?;
    let usage_5h = rl.five_hour.as_ref().and_then(|w| w.used_percentage);
    let usage_7d = rl.seven_day.as_ref().and_then(|w| w.used_percentage);
    if usage_5h.is_none() && usage_7d.is_none() {
        return None;
    }
    let resets_at_5h = rl.five_hour.as_ref().and_then(|w| w.resets_at);
    let resets_at_7d = rl.seven_day.as_ref().and_then(|w| w.resets_at);
    let now = chrono::Utc::now().timestamp();
    let usage = UsageInfo {
        usage_5h,
        usage_7d,
        reset_5h: reset_secs(resets_at_5h, now),
        reset_7d: reset_secs(resets_at_7d, now),
    };
    let cache = CachedRateLimits { usage_5h, usage_7d, resets_at_5h, resets_at_7d };
    Some((usage, cache))
}

fn usage_from_cache(cached: &CachedRateLimits) -> Option<UsageInfo> {
    if cached.usage_5h.is_none() && cached.usage_7d.is_none() {
        return None;
    }
    let now = chrono::Utc::now().timestamp();
    Some(UsageInfo {
        usage_5h: cached.usage_5h,
        usage_7d: cached.usage_7d,
        reset_5h: reset_secs(cached.resets_at_5h, now),
        reset_7d: reset_secs(cached.resets_at_7d, now),
    })
}

fn main() {
    let data = stdin::read_stdin();
    let config = config::load_config();
    let mut st = state::load_state(data.transcript_path.as_deref());
    let transcript_data = transcript::parse_transcript(data.transcript_path.as_deref(), &mut st);
    let git = git::read_git_info(data.cwd.as_deref(), st.git_index_mtime);
    let git_info = git.as_ref().map(|(info, _)| info);
    if let Some((_, mtime)) = &git {
        st.git_index_mtime = Some(*mtime);
    }
    let usage = match usage_from_stdin(&data) {
        Some((info, cache)) => {
            st.cached_rate_limits = Some(cache);
            Some(info)
        }
        None => st
            .cached_rate_limits
            .as_ref()
            .and_then(usage_from_cache)
            .or_else(|| {
                let global = state::load_global_rate_limits()?;
                let info = usage_from_cache(&global)?;
                st.cached_rate_limits = Some(global);
                Some(info)
            }),
    };
    let output = render::render(&data, &config, &transcript_data, git_info, usage.as_ref());
    println!("{output}");
    state::save_state(&mut st, data.transcript_path.as_deref());
}
