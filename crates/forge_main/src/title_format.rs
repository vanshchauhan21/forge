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
    use insta::assert_snapshot;
    use once_cell::sync::Lazy;
    use regex::Regex;

    use super::*;

    static ANSI_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\x1B\[([0-9]{1,2}(;[0-9]{1,2})*)?[m|K]").unwrap());

    fn strip_ansi(s: &str) -> String {
        ANSI_REGEX.replace_all(s, "").to_string()
    }

    #[test]
    fn test_format_title_various_lengths() {
        let test_cases = [
            ("", "empty"),
            ("A", "single_char"),
            ("Hello", "short"),
            ("Hello, World!", "medium"),
            (
                "This is a much longer title that should still work properly",
                "long",
            ),
            ("     ", "whitespace"),
        ];

        for (title, name) in test_cases {
            let formatted = format_title(title);
            let clean = strip_ansi(&formatted);
            assert_snapshot!(name, clean);
        }
    }

    #[test]
    fn test_border_alignment() {
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
    }
}
