#![feature(string_from_utf8_lossy_owned)]

use anyhow::Result;

mod actions;
mod cli;
mod terms;

fn main() -> Result<()> {
    cli::run()
}
