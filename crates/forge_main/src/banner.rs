use std::io;

use colored::Colorize;

const BANNER: &str = include_str!("banner");

pub fn display() -> io::Result<()> {
    // Split the banner into lines and display each line dimmed
    for line in BANNER.lines() {
        println!("{}", line.dimmed());
    }
    Ok(())
}
