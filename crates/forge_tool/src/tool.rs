use std::collections::BTreeSet;

use forge_domain::{ToolDefinition, ToolName};
use inflector::Inflector;
use schemars::schema::RootSchema;
use schemars::{schema_for, JsonSchema};
use serde_json::Value;

use crate::tool_call_service::{JsonTool, ToolCallService};
use crate::Description;

pub struct Tool {
    pub executable: Box<dyn ToolCallService<Input = Value, Output = Value> + Send + Sync + 'static>,
    pub definition: ToolDefinition,
}

impl Tool {
    pub fn new<T>(tool: T) -> Tool
    where
        T: ToolCallService + Description + Send + Sync + 'static,
        T::Input: serde::de::DeserializeOwned + JsonSchema,
        T::Output: serde::Serialize + JsonSchema,
    {
        let name = std::any::type_name::<T>()
            .split("::")
            .last()
            .unwrap()
            .to_snake_case();

        let executable = Box::new(JsonTool::new(tool));
        let input: RootSchema = schema_for!(T::Input);
        let output: RootSchema = schema_for!(T::Output);
        let mut description = T::description().to_string();

        description.push_str("\n\nParameters:");

        let required = input
            .schema
            .clone()
            .object
            .iter()
            .flat_map(|object| object.required.clone().into_iter())
            .collect::<BTreeSet<_>>();
        for (name, desc) in input
            .schema
            .object
            .clone()
            .into_iter()
            .flat_map(|object| object.properties.into_iter())
            .flat_map(|(name, props)| {
                props
                    .into_object()
                    .metadata
                    .into_iter()
                    .map(move |meta| (name.clone(), meta))
            })
            .flat_map(|(name, meta)| {
                meta.description
                    .into_iter()
                    .map(move |desc| (name.clone(), desc))
            })
        {
            description.push_str("\n- ");
            description.push_str(&name);

            if required.contains(&name) {
                description.push_str(" (required)");
            }

            description.push_str(": ");
            description.push_str(&desc);
        }

        let definition = ToolDefinition {
            name: ToolName::new(name.clone()),
            description,
            input_schema: input,
            output_schema: Some(output),
        };

        Tool { executable, definition }
    }
}
