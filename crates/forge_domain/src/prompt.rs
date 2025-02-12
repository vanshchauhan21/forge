use std::path::PathBuf;

use handlebars::Handlebars;
use serde::{Deserialize, Serialize};

use crate::Error;

pub enum PromptContent {
    Text(String),
    File(PathBuf),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Prompt<V> {
    pub template: PromptTemplate,
    #[serde(skip_serializing_if = "Schema::is_empty")]
    pub variables: Schema<V>,
}

impl<V> Prompt<V> {
    pub fn new(template: impl ToString) -> Self {
        Self {
            template: PromptTemplate(template.to_string()),
            variables: Schema { _marker: std::marker::PhantomData },
        }
    }
}

impl<V: Serialize> Prompt<V> {
    /// Register all known partial templates with the Handlebars registry
    fn register_partials(hb: &mut Handlebars) {
        // Register all partial templates. Template names must match the file names
        // without .mustache extension
        const PARTIALS: &[(&str, &str)] = &[
            (
                "tool-usage-examples",
                include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../templates/partials/tool-usage-examples.mustache"
                )),
            ),
            (
                "agent-tools",
                include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../templates/partials/agent-tools.mustache"
                )),
            ),
        ];

        for (name, content) in PARTIALS {
            let _ = hb.register_partial(name, content);
        }
    }

    pub fn render(&self, ctx: &V) -> crate::Result<String> {
        let mut hb = Handlebars::new();
        hb.set_strict_mode(true);
        hb.register_escape_fn(|str| str.to_string());

        // Register all partial templates
        Self::register_partials(&mut hb);

        hb.render_template(self.template.as_str(), &ctx)
            .map_err(Error::Template)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema<S> {
    #[serde(skip)]
    _marker: std::marker::PhantomData<S>,
}

impl<S> Schema<S> {
    pub fn is_empty(&self) -> bool {
        true // Since we skip the only field (_marker), this is always empty
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PromptTemplate(String);

impl PromptTemplate {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}
