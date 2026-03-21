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
    let output = render::render(&data, &config, &transcript_data);
    println!("{output}");
    state::save_state(&st, data.transcript_path.as_deref());
}
