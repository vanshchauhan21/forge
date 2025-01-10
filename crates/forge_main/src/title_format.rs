use colored::Colorize;
use unicode_width::UnicodeWidthStr;

/// Create a border line with the given width and fill character
pub fn make_border_line(width: usize, is_top: bool) -> String {
    let left = if is_top { '╔' } else { '╚' };
    let right = if is_top { '╗' } else { '╝' };
    let fill = '═'.to_string().repeat(width);
    format!("{}{}{}", left, fill, right)
}

/// Formats and returns the title with decorative borders
pub fn format_title(title: &str) -> String {
    let title_width = title.width();
    let colored_title = title.bright_cyan().bold();

    // For empty titles, use a fixed minimal width
    if title_width == 0 {
        return String::from("╔══════╗\n║      ║\n╚══════╝");
    }

    let border_width = title_width + 2; // Add 2 for the side spaces
    let top = make_border_line(border_width, true);
    let bottom = make_border_line(border_width, false);
    let middle = format!("║ {} ║", colored_title);

    format!("{}\n{}\n{}", top, middle, bottom)
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;
    use once_cell::sync::Lazy;
    use regex::Regex;

    static ANSI_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\x1B\[([0-9]{1,2}(;[0-9]{1,2})*)?[m|K]").unwrap());

    fn strip_ansi(s: &str) -> String {
        ANSI_REGEX.replace_all(s, "").to_string()
    }

    fn debug_string_chars(s: &str) {
        println!("String literal bytes: {:?}", s.as_bytes());
        println!("String visual width: {}", s.width());
        println!("String chars:");
        for (i, c) in s.chars().enumerate() {
            println!("  {}: {:?} (width: {})", i, c, c.to_string().width());
        }
    }

    #[test]
    fn test_format_title_various_lengths() {
        // Test cases
        let test_cases = [
            ("", "empty"),
            ("A", "single_char"),
            ("Hello", "short"),
            ("Hello, World!", "medium"),
            (
                "This is a much longer title that should still work properly",
                "long",
            ),
            ("     ", "whitespace"), // Added whitespace test
        ];

        for (title, name) in test_cases {
            let formatted = format_title(title);
            // Strip ANSI codes for snapshot testing
            let clean = strip_ansi(&formatted);
            println!("\nTest case: {}", name);
            println!("Input: {:?}", title);
            println!("Output:\n{}", clean);
            println!("Character analysis of output:");
            debug_string_chars(&clean);
            assert_snapshot!(name, clean);
        }
    }

    #[test]
    fn test_border_alignment() {
        // Test cases with expected alignments
        let cases = [
            "",
            "Test Title",
            "Unicode: 👋",
            "     ",
            "A very long title that should still work correctly",
        ];

        for title in cases {
            let formatted = strip_ansi(&format_title(title));
            let lines: Vec<&str> = formatted.lines().collect();

            println!("\nTesting title: {:?}", title);
            println!("Output:\n{}", formatted);
            println!(
                "Line visual widths: {:?}",
                lines.iter().map(|l| l.width()).collect::<Vec<_>>()
            );
            println!("Line character analysis:");
            for (i, line) in lines.iter().enumerate() {
                println!("Line {}:", i);
                debug_string_chars(line);
            }

            assert_eq!(lines.len(), 3, "Should have exactly 3 lines");
            assert_eq!(
                lines[0].width(),
                lines[1].width(),
                "Top and middle lines should be same width for title: {:?}\nLines:\n{:?}",
                title,
                lines
            );
            assert_eq!(
                lines[1].width(),
                lines[2].width(),
                "Middle and bottom lines should be same width for title: {:?}\nLines:\n{:?}",
                title,
                lines
            );
        }
    }

    #[test]
    fn test_make_border_line() {
        assert_eq!(make_border_line(4, true), "╔════╗");
        assert_eq!(make_border_line(4, false), "╚════╝");
        assert_eq!(make_border_line(0, true), "╔╗");
        assert_eq!(make_border_line(0, false), "╚╝");

        println!("\nDebug border line characters:");
        let line = make_border_line(4, true);
        debug_string_chars(&line);
    }
}