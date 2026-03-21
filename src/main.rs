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
    let output = render::render(&data, &config);
    println!("{output}");
}
