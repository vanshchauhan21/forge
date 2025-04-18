use std::fmt::{self, Display, Formatter};

use colored::Colorize;
use convert_case::{Case, Casing};
use derive_setters::Setters;

#[derive(Clone, Setters)]
#[setters(into, strip_option)]
pub struct TitleFormat {
    pub title: String,
    pub sub_title: Option<String>,
    pub error: Option<String>,
}

pub trait TitleExt {
    fn title_fmt(&self) -> TitleFormat;
}

impl<T> TitleExt for T
where
    T: Into<TitleFormat> + Clone,
{
    fn title_fmt(&self) -> TitleFormat {
        self.clone().into()
    }
}

impl TitleFormat {
    /// Create a status for executing a tool
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            error: None,
            sub_title: Default::default(),
        }
    }
    pub fn format(&self) -> String {
        let mut buf = String::new();
        buf.push_str(format!("{} ", "⏺".blue()).as_str());
        let mut title = self.title.to_case(Case::Title).white().bold();

        if self.error.is_some() {
            title = title.red().bold();
        }

        buf.push_str(&format!("{}", title));

        if let Some(ref sub_title) = self.sub_title {
            buf.push_str(&format!(" {}", sub_title).to_string());
        }

        if let Some(ref error) = self.error {
            buf.push_str(&format!(" {}", error).to_string());
        }

        buf
    }
}

impl Display for TitleFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}
