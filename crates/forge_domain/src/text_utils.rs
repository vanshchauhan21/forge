/// Extracts content between the specified XML-style tags
///
/// # Arguments
///
/// * `text` - The text to extract content from
/// * `tag_name` - The name of the XML tag (without angle brackets)
///
/// # Returns
///
/// * `Some(&str)` containing the extracted content if tags are found
/// * `None` if the tags are not found
///
/// # Example
///
/// ```
/// let text = "Some text <summary>This is the important part</summary> and more text";
/// let extracted = extract_tag_content(text, "summary");
/// assert_eq!(extracted, Some("This is the important part"));
/// ```
pub fn extract_tag_content<'a>(text: &'a str, tag_name: &str) -> Option<&'a str> {
    let opening_tag = format!("<{}>", tag_name);
    let closing_tag = format!("</{}>", tag_name);

    if let Some(start_idx) = text.find(&opening_tag) {
        if let Some(end_idx) = text.find(&closing_tag) {
            let content_start = start_idx + opening_tag.len();
            if content_start < end_idx {
                return Some(text[content_start..end_idx].trim());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_extract_tag_content() {
        let fixture = "Some text <summary>This is the important part</summary> and more text";
        let actual = extract_tag_content(fixture, "summary");
        let expected = Some("This is the important part");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_extract_tag_content_no_tags() {
        let fixture = "Some text without any tags";
        let actual = extract_tag_content(fixture, "summary");
        let expected = None;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_extract_tag_content_with_different_tag() {
        let fixture = "Text with <custom>Custom content</custom> tags";
        let actual = extract_tag_content(fixture, "custom");
        let expected = Some("Custom content");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_extract_tag_content_with_malformed_tags() {
        let fixture = "Text with <opening> but no closing tag";
        let actual = extract_tag_content(fixture, "opening");
        let expected = None;
        assert_eq!(actual, expected);
    }
}
