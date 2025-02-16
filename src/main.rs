#![feature(string_from_utf8_lossy_owned)]

use anyhow::Result;

mod actions;
mod cli;

fn main() -> Result<()> {
    cli::run()
}
