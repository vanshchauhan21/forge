use std::io;

use colored::Colorize;

const BANNER: &str = include_str!("banner");

pub fn display() -> io::Result<()> {
    let commands = ["/info", "/help"];
    println!("{} {}", BANNER.dimmed(), commands.join(", ").bold());
    Ok(())
}
