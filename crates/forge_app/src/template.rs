use forge_domain::{Template, TemplateService};
use handlebars::Handlebars;
use rust_embed::Embed;
use serde::Serialize;

#[derive(Embed)]
#[folder = "../../templates/"]
struct Templates;

pub struct ForgeTemplateService {
    hb: Handlebars<'static>,
}

impl Default for ForgeTemplateService {
    fn default() -> Self {
        Self::new()
    }
}

impl ForgeTemplateService {
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
impl TemplateService for ForgeTemplateService {
    async fn render<T: Serialize + Send + Sync>(
        &self,
        prompt: &Template<T>,
        ctx: &T,
    ) -> anyhow::Result<String> {
        Ok(self.hb.render_template(prompt.template.as_str(), ctx)?)
    }
}
