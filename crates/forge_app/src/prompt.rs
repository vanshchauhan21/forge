use forge_domain::{Prompt, PromptService};
use handlebars::Handlebars;
use rust_embed::Embed;
use serde::Serialize;

#[derive(Embed)]
#[folder = "../../templates/"]
struct Templates;

pub struct ForgePromptService {
    hb: Handlebars<'static>,
}

impl Default for ForgePromptService {
    fn default() -> Self {
        Self::new()
    }
}

impl ForgePromptService {
    pub fn new() -> Self {
        let mut hb = Handlebars::new();
        hb.set_strict_mode(true);
        hb.register_escape_fn(|str| str.to_string());

        // Register all partial templates
        hb.register_embed_templates::<Templates>().unwrap();

        Self { hb }
    }
}

#[async_trait::async_trait]
impl PromptService for ForgePromptService {
    async fn render<T: Serialize + Send + Sync>(
        &self,
        prompt: &Prompt<T>,
        ctx: &T,
    ) -> anyhow::Result<String> {
        Ok(self.hb.render_template(prompt.template.as_str(), ctx)?)
    }
}
