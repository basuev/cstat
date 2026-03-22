mod config;
mod git;
mod render;
mod state;
mod stdin;
mod transcript;
mod types;

use types::UsageInfo;

fn usage_from_stdin(data: &types::StdinData) -> Option<UsageInfo> {
    let rl = data.rate_limits.as_ref()?;
    let usage_5h = rl.five_hour.as_ref().and_then(|w| w.used_percentage);
    let usage_7d = rl.seven_day.as_ref().and_then(|w| w.used_percentage);
    if usage_5h.is_none() && usage_7d.is_none() {
        return None;
    }
    let now = chrono::Utc::now().timestamp();
    let reset_5h = rl
        .five_hour
        .as_ref()
        .and_then(|w| w.resets_at)
        .map(|t| t - now)
        .filter(|&r| r > 0);
    Some(UsageInfo {
        usage_5h,
        usage_7d,
        reset_5h,
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
    let usage = usage_from_stdin(&data);
    let output = render::render(&data, &config, &transcript_data, git_info, usage.as_ref());
    println!("{output}");
    state::save_state(&st, data.transcript_path.as_deref());
}
