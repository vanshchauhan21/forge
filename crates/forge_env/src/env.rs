use crate::Result;
use handlebars::Handlebars;
use serde::Serialize;

#[derive(Serialize)]
struct EnvironmentValue {
    operating_system: String,
    current_working_dir: String,
}

pub struct Environment;

impl Environment {
    pub fn render(template: &str) -> Result<String> {
        let env = EnvironmentValue {
            operating_system: std::env::consts::OS.to_string(),
            current_working_dir: format!("{}", std::env::current_dir()?.display()),
        };

        let mut hb = Handlebars::new();
        hb.set_strict_mode(true);
        Ok(hb.render_template(template, &env)?)
    }
}
