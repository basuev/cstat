mod config;
mod git;
mod render;
mod state;
mod stdin;
mod transcript;
mod types;
mod usage;

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
    let output = render::render(&data, &config, &transcript_data, git_info);
    println!("{output}");
    state::save_state(&st, data.transcript_path.as_deref());
}
