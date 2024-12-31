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
    // Skip any leading text/whitespace until we find a tool tag
    let (input, _) = take_until("<")(input)?;
    let (input, _) = tag("<")(input)?;
    let (input, name) = parse_identifier(input)?;
    let (input, _) = tag(">")(input)?;
    let (input, args) = parse_args(input)?;
    let (input, _) = tag("</")(input)?;
    let (input, _) = tag(name)(input)?;
    let (input, _) = tag(">")(input)?;

    Ok((input, ToolCallParsed { name: name.to_string(), args }))
}

pub fn parse(input: &str) -> Result<ToolCall, String> {
    let (_, parsed) = parse_tool_call(input).map_err(|e| format!("Parsing Error: {}", e))?;

    Ok(ToolCall {
        name: ToolName::new(parsed.name),
        call_id: None,
        arguments: Value::Object(parsed.args.into_iter().fold(
            serde_json::Map::new(),
            |mut map, (key, value)| {
                map.insert(key, Value::String(value));
                map
            },
        )),
    })
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
            for (key, value) in &self.args {
                xml.push_str(&format!("<{}>{}</{}>", key, value, key));
            }
            xml.push_str(&format!("</{}>", self.name));
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
        let expected = tool.build_expected();
        assert_eq!(action, expected);
    }

    #[test]
    fn test_parse_with_surrounding_text() {
        let tool = ToolCallBuilder::new("tool_name").arg("arg1", "value1");
        let input = format!("Some text {} more text", tool.build_xml());

        let action = parse(&input).unwrap();
        let expected = tool.build_expected();
        assert_eq!(action, expected);
    }

    #[test]
    fn test_parse_multiple_tool_calls() {
        let tool1 = ToolCallBuilder::new("tool1").arg("arg1", "value1");
        let tool2 = ToolCallBuilder::new("tool2").arg("arg2", "value2");
        let input = format!("{} Some text {}", tool1.build_xml(), tool2.build_xml());

        let action = parse(&input).unwrap();
        let expected = tool1.build_expected();
        assert_eq!(action, expected);

        let second_tool_input = &input[input.find("<tool2>").unwrap()..];
        let action2 = parse(second_tool_input).unwrap();
        let expected2 = tool2.build_expected();
        assert_eq!(action2, expected2);
    }

    #[test]
    fn test_parse_empty_args() {
        let tool = ToolCallBuilder::new("tool_name");

        let action = parse(&tool.build_xml()).unwrap();
        let expected = tool.build_expected();
        assert_eq!(action, expected);
    }

    #[test]
    fn test_parse_with_special_chars() {
        let tool = ToolCallBuilder::new("tool_name")
            .arg("arg1", "value with spaces")
            .arg("arg2", "value&with#special@chars");

        let action = parse(&tool.build_xml()).unwrap();
        let expected = tool.build_expected();
        assert_eq!(action, expected);
    }
}
