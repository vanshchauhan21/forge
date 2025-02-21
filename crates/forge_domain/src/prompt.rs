use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Prompt<V> {
    pub template: PromptTemplate,
    #[serde(flatten)]
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

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct PromptTemplate(String);

impl PromptTemplate {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}
