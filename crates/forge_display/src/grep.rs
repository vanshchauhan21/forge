use std::collections::BTreeMap;

use console::style;
use regex::Regex;

/// RipGrepFormatter formats search results in ripgrep-like style.
#[derive(Clone)]
pub struct GrepFormat(Vec<String>);

/// Represents a parsed line from grep-like output format
/// (path:line_num:content)
#[derive(Debug)]
struct ParsedLine<'a> {
    /// File path where the match was found
    path: &'a str,
    /// Line number of the match
    line_num: &'a str,
    /// Content of the matching line
    content: &'a str,
}

impl<'a> ParsedLine<'a> {
    /// Parse a line in the format "path:line_num:content"
    ///
    /// # Arguments
    /// * `line` - The line to parse in the format "path:line_num:content"
    ///
    /// # Returns
    /// * `Some(ParsedLine)` if the line matches the expected format
    /// * `None` if the line is malformed
    pub fn parse(line: &'a str) -> Option<Self> {
        let parts: Vec<_> = line.split(':').collect();
        if parts.len() != 3 {
            return None;
        }

        // Validate that path and line number parts are not empty
        // and that line number contains only digits
        if parts[0].is_empty()
            || parts[1].is_empty()
            || !parts[1].chars().all(|c| c.is_ascii_digit())
        {
            return None;
        }

        Some(Self {
            path: parts[0].trim(),
            line_num: parts[1].trim(),
            content: parts[2].trim(),
        })
    }
}

type Lines<'a> = Vec<(&'a str, &'a str)>;
impl GrepFormat {
    pub fn new(lines: Vec<String>) -> Self {
        Self(lines)
    }

    /// Collect file entries and determine the maximum line number width
    fn collect_entries(lines: &[String]) -> (BTreeMap<&str, Lines>, usize) {
        lines
            .iter()
            .map(String::as_str)
            .filter_map(ParsedLine::parse)
            .fold((BTreeMap::new(), 0), |(mut entries, max_width), parsed| {
                let new_width = max_width.max(parsed.line_num.len());
                entries
                    .entry(parsed.path)
                    .or_default()
                    .push((parsed.line_num, parsed.content));
                (entries, new_width)
            })
    }

    /// Format a single line with colorization and consistent padding
    fn format_line(num: &str, content: &str, regex: &Regex, padding: usize) -> String {
        let num = style(format!("{:>padding$}: ", num, padding = padding)).dim();

        // Format the content with highlighting
        let line = regex.find(content).map_or_else(
            || content.to_string(),
            |mat| {
                format!(
                    "{}{}{}",
                    &content[..mat.start()],
                    style(&content[mat.start()..mat.end()]).yellow().bold(),
                    &content[mat.end()..]
                )
            },
        );

        format!("{}{}\n", num, line)
    }

    /// Format a group of lines for a single file
    fn format_file_group(
        path: &str,
        group: Vec<(&str, &str)>,
        regex: &Regex,
        max_num_width: usize,
    ) -> String {
        let file_header = style(path).cyan();
        let formatted_lines = group
            .into_iter()
            .map(|(num, content)| Self::format_line(num, content, regex, max_num_width))
            .collect::<String>();
        format!("{}\n{}", file_header, formatted_lines)
    }

    /// Format search results with colorized output grouped by path
    pub fn format(&self, regex: &Regex) -> String {
        if self.0.is_empty() {
            return String::new();
        }

        // First pass: collect entries and find max width
        let (entries, max_num_width) = Self::collect_entries(&self.0);

        // Print the results on separate lines
        let formatted_entries: Vec<_> = entries
            .into_iter()
            .map(|(path, group)| Self::format_file_group(path, group, regex, max_num_width))
            .collect();

        // Join all results with newlines
        formatted_entries.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::{Display, Formatter};

    use insta::assert_snapshot;

    use super::*;

    /// Specification for a grep format test case
    #[derive(Debug)]
    struct GrepSpec {
        description: String,
        input: Vec<String>,
        output: String,
    }

    impl GrepSpec {
        /// Create a new test specification with computed fields
        fn new(description: &str, input: Vec<&str>, pattern: &str) -> Self {
            let input: Vec<String> = input.iter().map(|s| s.to_string()).collect();

            // Generate the formatted output
            let formatter = GrepFormat::new(input.clone());
            let output =
                strip_ansi_escapes::strip_str(formatter.format(&Regex::new(pattern).unwrap()))
                    .to_string();

            Self { description: description.to_string(), input, output }
        }
    }

    impl Display for GrepSpec {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            writeln!(f, "\n[{}]", self.description)?;
            writeln!(f, "[RAW]")?;
            writeln!(f, "{}", self.input.join("\n"))?;
            writeln!(f, "[FMT]")?;
            writeln!(f, "{}", self.output)
        }
    }

    #[derive(Default, Debug)]
    struct GrepSuite(Vec<GrepSpec>);

    impl GrepSuite {
        fn add(&mut self, description: &str, input: Vec<&str>, pattern: &str) {
            self.0.push(GrepSpec::new(description, input, pattern));
        }
    }

    impl Display for GrepSuite {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            for spec in &self.0 {
                writeln!(f, "{}", spec)?;
            }
            Ok(())
        }
    }

    #[test]
    fn test_combined_grep_suite() {
        let mut suite = GrepSuite::default();

        suite.add(
            "Basic single file with two matches",
            vec!["file.txt:1:first match", "file.txt:2:second match"],
            "match",
        );

        suite.add(
            "Multiple files with various matches",
            vec![
                "file1.txt:1:match in file1",
                "file2.txt:1:first match in file2",
                "file2.txt:2:second match in file2",
                "file3.txt:1:match in file3",
            ],
            "file",
        );

        suite.add(
            "File with varying line number widths",
            vec![
                "file.txt:1:first line",
                "file.txt:5:fifth line",
                "file.txt:10:tenth line",
                "file.txt:100:hundredth line",
            ],
            "line",
        );

        suite.add(
            "Mix of valid and invalid input lines",
            vec![
                "file.txt:1:valid match",
                "malformed line without separator",
                "file.txt:2:another valid match",
            ],
            "match",
        );

        suite.add("Empty input vector", vec![], "anything");

        suite.add(
            "Input with special characters and formatting",
            vec![
                "path/to/file.txt:1:contains 🦀 rust",
                "path/to/file.txt:2:has\ttabs\tand\tspaces",
                "path/to/file.txt:3:contains\nnewlines",
            ],
            "contains",
        );

        suite.add(
            "Multiple files with same line numbers",
            vec![
                "test1.rs:10:fn test1()",
                "test2.rs:10:fn test2()",
                "test3.rs:10:fn test3()",
            ],
            "fn",
        );

        suite.add(
            "Content with full-width unicode characters",
            vec![
                "test.txt:1:Contains 你好 characters",
                "test.txt:2:More UTF-8 ありがとう here",
            ],
            "Contains",
        );

        assert_snapshot!(suite);
    }
}
