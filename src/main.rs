use clap::Parser;

use crate::{engine::Engine, runner::file_runner::FileRunner};

mod engine;
mod models;
mod runner;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    input_file: String,
}

fn main() {
    let args = Args::parse();

    let mut engine = Engine::new();

    let runner = FileRunner::new();

    runner
        .run(&args.input_file, &mut engine)
        .expect("Error occured running the engine");
}
