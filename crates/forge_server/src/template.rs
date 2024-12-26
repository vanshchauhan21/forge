use std::fmt::Display;

use forge_provider::{AnyMessage, Message};

#[derive(Debug, Clone)]
pub struct Tag {
    // TODO: move to enum type
    name: String,
    attributes: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub enum MessageTemplate {
    Tagged {
        tag: Tag,
        content: String,
    },
    Combine {
        left: Box<MessageTemplate>,
        right: Box<MessageTemplate>,
    },
}

impl MessageTemplate {
    fn new(tag: Tag, content: String) -> Self {
        Self::Tagged { tag, content }
    }

    pub fn task(content: String) -> Self {
        let tag = Tag { name: "task".to_string(), attributes: vec![] };

        Self::new(tag, content)
    }

    pub fn file(path: String, content: String) -> Self {
        let tag = Tag {
            name: "file_content".to_string(),
            attributes: vec![("path".to_string(), path)],
        };

        Self::new(tag, content)
    }

    pub fn append(self, other: Self) -> Self {
        Self::Combine { left: Box::new(self), right: Box::new(other) }
    }
}

impl Display for MessageTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageTemplate::Tagged { tag, content } => {
                let tag_name = tag.name.to_uppercase();
                f.write_str("<")?;
                f.write_str(&tag_name)?;

                for (key, value) in &tag.attributes {
                    f.write_str(" ")?;
                    f.write_str(key)?;
                    f.write_str("=\"")?;
                    f.write_str(value)?;
                    f.write_str("\"")?;
                }

                f.write_str(">")?;

                f.write_str(content)?;

                f.write_str("<")?;
                f.write_str(&tag_name)?;
                f.write_str("/>")?;
            }
            MessageTemplate::Combine { left, right } => {
                write!(f, "{}\n{}", left, right)?;
            }
        }

        Ok(())
    }
}

impl From<MessageTemplate> for AnyMessage {
    fn from(value: MessageTemplate) -> Self {
        Message::user(value.to_string()).into()
    }
}
