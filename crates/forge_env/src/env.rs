use crate::Result;
use handlebars::Handlebars;
use serde::Serialize;

#[derive(Serialize)]
pub struct Environment {
    operating_system: String,
    current_working_dir: String,
}

impl Environment {
    pub fn build() -> Result<Self> {
        Ok(Self {
            operating_system: std::env::consts::OS.to_string(),
            current_working_dir: format!("{}", std::env::current_dir()?.display()),
        })
    }

    pub fn render(&self, template: &str) -> Result<String> {
        let mut hb = Handlebars::new();
        hb.set_strict_mode(true);
        Ok(hb.render_template(template, self)?)
    }
}
