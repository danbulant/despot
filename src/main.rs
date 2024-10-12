use clap::Parser;
use cli::Args;
use cushy::{PendingApp, Run, TokioRuntime};

mod vibrancy;
mod theme;
mod cli;

fn main() -> cushy::Result {
    let args = Args::parse();
    let mut app = PendingApp::new(TokioRuntime::default());

    app.run()
}