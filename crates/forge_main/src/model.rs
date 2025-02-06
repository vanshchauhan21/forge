use std::collections::BTreeMap;

use forge_domain::Model;

use crate::info::Info;

fn humanize_context_length(length: u64) -> String {
    if length >= 1_000_000 {
        format!("{:.1}M context", length as f64 / 1_000_000.0)
    } else if length >= 1_000 {
        format!("{:.1}K context", length as f64 / 1_000.0)
    } else {
        format!("{} context", length)
    }
}

impl From<&[Model]> for Info {
    fn from(models: &[Model]) -> Self {
        let mut info = Info::new();

        let mut models_by_provider: BTreeMap<String, Vec<&Model>> = BTreeMap::new();
        for model in models {
            let provider = model
                .id
                .as_str()
                .split('/')
                .next()
                .unwrap_or("unknown")
                .to_string();
            models_by_provider.entry(provider).or_default().push(model);
        }

        for (provider, provider_models) in models_by_provider.iter() {
            info = info.add_title(provider.to_string());
            for model in provider_models {
                info = info.add_item(
                    &model.name,
                    format!(
                        "{} ({})",
                        model.id,
                        humanize_context_length(model.context_length)
                    ),
                );
            }
        }

        info
    }
}
