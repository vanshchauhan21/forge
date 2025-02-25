use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct Template<V> {
    pub template: String,
    _marker: std::marker::PhantomData<V>,
}

impl<V> Template<V> {
    pub fn new(template: impl ToString) -> Self {
        Self {
            template: template.to_string(),
            _marker: std::marker::PhantomData,
        }
    }
}
