use std::collections::HashMap;

use forge_tool::ToolName;
use nom::bytes::complete::{tag, take_until, take_while1};
use nom::character::complete::multispace0;
use nom::multi::many0;
use nom::{IResult, Parser};
use serde_json::Value;

use super::ToolCall;

#[derive(Debug, PartialEq)]
pub struct ToolCallParsed {
    pub name: String,
    pub args: HashMap<String, String>,
}

// Allow alphanumeric and underscore characters
fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn parse_identifier(input: &str) -> IResult<&str, &str> {
    take_while1(is_identifier_char)(input)
}

fn parse_arg(input: &str) -> IResult<&str, (&str, &str)> {
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("<")(input)?;
    let (input, key) = parse_identifier(input)?;
    let (input, _) = tag(">")(input)?;
    let (input, value) = take_until("</")(input)?;
    let (input, _) = tag("</")(input)?;
    let (input, _) = tag(key)(input)?;
    let (input, _) = tag(">")(input)?;
    let (input, _) = multispace0(input)?;

    Ok((input, (key, value)))
}

fn parse_args(input: &str) -> IResult<&str, HashMap<String, String>> {
    let mut arg_parser = many0(parse_arg);
    let (input, args) = arg_parser.parse(input)?;
    let mut map = HashMap::new();
    for (key, value) in args {
        map.insert(key.to_string(), value.to_string());
    }
    Ok((input, map))
}

fn parse_tool_call(input: &str) -> IResult<&str, ToolCallParsed> {
    let (input, _) = tag("<")(input)?;
    let (input, name) = parse_identifier(input)?;
    let (input, _) = tag(">")(input)?;
    let (input, args) = parse_args(input)?;
    let (input, _) = tag("</")(input)?;
    let (input, _) = tag(name)(input)?;
    let (input, _) = tag(">")(input)?;

    Ok((input, ToolCallParsed { name: name.to_string(), args }))
}

fn find_next_tool_call(input: &str) -> IResult<&str, &str> {
    // Find the next occurrence of a tool call opening tag
    let (remaining, _) = take_until("<")(input)?;
    Ok((remaining, ""))
}

fn tool_call_to_struct(parsed: ToolCallParsed) -> ToolCall {
    ToolCall {
        name: ToolName::new(parsed.name),
        call_id: None,
        arguments: Value::Object(parsed.args.into_iter().fold(
            serde_json::Map::new(),
            |mut map, (key, value)| {
                map.insert(key, Value::String(value));
                map
            },
        )),
    }
}

pub fn parse(input: &str) -> Result<Vec<ToolCall>, String> {
    let mut tool_calls = Vec::new();
    let mut current_input = input;

    while !current_input.is_empty() {
        // Try to find the next tool call
        match find_next_tool_call(current_input) {
            Ok((remaining, _)) => {
                // Try to parse a tool call at the current position
                match parse_tool_call(remaining) {
                    Ok((new_remaining, parsed)) => {
                        tool_calls.push(tool_call_to_struct(parsed));
                        current_input = new_remaining;
                    }
                    Err(e) => {
                        if tool_calls.is_empty() {
                            return Err(format!("Failed to parse tool call: {}", e));
                        }
                        // If we've already found some tool calls, we can stop here
                        break;
                    }
                }
            }
            Err(_) => break, // No more tool calls found
        }
    }

    if tool_calls.is_empty() {
        Err("No valid tool calls found in input".to_string())
    } else {
        Ok(tool_calls)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test helpers
    struct ToolCallBuilder {
        name: String,
        args: HashMap<String, String>,
    }

    impl ToolCallBuilder {
        fn new(name: &str) -> Self {
            Self { name: name.to_string(), args: HashMap::new() }
        }

        fn arg(mut self, key: &str, value: &str) -> Self {
            self.args.insert(key.to_string(), value.to_string());
            self
        }

        fn build_xml(&self) -> String {
            let mut xml = format!("<{}>", self.name);
            let args: Vec<_> = self.args.iter().collect();
            for (idx, (key, value)) in args.iter().enumerate() {
                xml.push_str(&format!(
                    "<{}>{}</{}>{}",
                    key,
                    value,
                    key,
                    if idx < args.len() - 1 { " " } else { "" }
                ));
            }
            xml.push_str(&format!("</{}>\n", self.name));
            xml
        }

        fn build_expected(&self) -> ToolCall {
            let mut args = Value::Object(Default::default());
            for (key, value) in &self.args {
                args.as_object_mut()
                    .unwrap()
                    .insert(key.clone(), Value::String(value.clone()));
            }
            ToolCall {
                name: ToolName::new(&self.name),
                call_id: None,
                arguments: args,
            }
        }
    }

    #[test]
    fn test_parse_arg() {
        let action = parse_arg("<key>value</key>").unwrap();
        let expected = ("", ("key", "value"));
        assert_eq!(action, expected);
    }

    #[test]
    fn test_parse_args() {
        let action = parse_args("<key1>value1</key1> <key2>value2</key2>")
            .unwrap()
            .1;
        let expected = {
            let mut map = HashMap::new();
            map.insert("key1".to_string(), "value1".to_string());
            map.insert("key2".to_string(), "value2".to_string());
            map
        };
        assert_eq!(action, expected);
    }

    #[test]
    fn test_parse_tool_call() {
        let tool = ToolCallBuilder::new("tool_name")
            .arg("arg1", "value1")
            .arg("arg2", "value2");

        let action = parse_tool_call(&tool.build_xml()).unwrap().1;
        let expected = ToolCallParsed { name: "tool_name".to_string(), args: tool.args };
        assert_eq!(action, expected);
    }

    #[test]
    fn test_parse() {
        let tool = ToolCallBuilder::new("tool_name")
            .arg("arg1", "value1")
            .arg("arg2", "value2");

        let action = parse(&tool.build_xml()).unwrap();
        let expected = vec![tool.build_expected()];
        assert_eq!(action, expected);
    }

    #[test]
    fn test_parse_with_surrounding_text() {
        let tool = ToolCallBuilder::new("tool_name").arg("arg1", "value1");
        let input = format!("Some text {} more text", tool.build_xml());

        let action = parse(&input).unwrap();
        let expected = vec![tool.build_expected()];
        assert_eq!(action, expected);
    }

    #[test]
    fn test_parse_multiple_tool_calls() {
        let tool1 = ToolCallBuilder::new("tool1").arg("arg1", "value1");
        let tool2 = ToolCallBuilder::new("tool2").arg("arg2", "value2");
        let input = format!("{} Some text {}", tool1.build_xml(), tool2.build_xml());

        let action = parse(&input).unwrap();
        let expected = vec![tool1.build_expected(), tool2.build_expected()];
        assert_eq!(action, expected);
    }

    #[test]
    fn test_parse_empty_args() {
        let tool = ToolCallBuilder::new("tool_name");

        let action = parse(&tool.build_xml()).unwrap();
        let expected = vec![tool.build_expected()];
        assert_eq!(action, expected);
    }

    #[test]
    fn test_parse_with_special_chars() {
        let tool = ToolCallBuilder::new("tool_name")
            .arg("arg1", "value with spaces")
            .arg("arg2", "value&with#special@chars");

        let action = parse(&tool.build_xml()).unwrap();
        let expected = vec![tool.build_expected()];
        assert_eq!(action, expected);
    }

    #[test]
    fn test_parse_with_large_text_between() {
        let tool1 = ToolCallBuilder::new("tool1").arg("arg1", "value1");
        let tool2 = ToolCallBuilder::new("tool2").arg("arg2", "value2");
        let input = format!(
            "{}\nLots of text here...\nMore text...\nEven more text...\n{}",
            tool1.build_xml(),
            tool2.build_xml()
        );

        let action = parse(&input).unwrap();
        let expected = vec![tool1.build_expected(), tool2.build_expected()];
        assert_eq!(action, expected);
    }
}
