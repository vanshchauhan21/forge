use forge_domain::{
    Agent, AgentBuilder, AgentId, Environment, NamedTool, Prompt, SystemContext, Variables,
    Workflow, WriteVariable,
};

const TITLE_GENERATOR_TEMPLATE: &str =
    include_str!("../../../templates/roles/title-generator.mustache");
const SOFTWARE_ENGINEER_TEMPLATE: &str =
    include_str!("../../../templates/roles/software-engineer.mustache");

use forge_tool::tools;

#[derive(Clone)]
pub struct ForgeWorkflow {
    pub title_agent: Agent,
    pub developer_agent: Agent,
}

impl ForgeWorkflow {
    pub fn new(env: Environment) -> Self {
        let agent = AgentBuilder::default().entry(true);

        let title_agent = agent
            .clone()
            .id(AgentId::new("title"))
            .model(env.small_model_id.clone())
            .description("Generates a title for the provided user task")
            .user_prompt(Prompt::<Variables>::new(
                "<technical_content>{{task}}</technical_content>",
            ))
            .system_prompt(Prompt::<SystemContext>::new(TITLE_GENERATOR_TEMPLATE))
            .max_turns(1u64)
            .tools(vec![WriteVariable::tool_name()]);

        let developer_agent = agent
            .clone()
            .id(AgentId::new("developer"))
            .model(env.large_model_id.clone())
            .ephemeral(false)
            .description("Does all the engineering tasks provided by the user")
            .user_prompt(Prompt::<Variables>::new("<task>{{task}}</task>"))
            .system_prompt(Prompt::<SystemContext>::new(SOFTWARE_ENGINEER_TEMPLATE))
            .tools(
                tools(&env)
                    .iter()
                    .map(|t| t.definition.name.clone())
                    .collect::<Vec<_>>(),
            );

        Self {
            title_agent: title_agent.build().unwrap(),
            developer_agent: developer_agent.build().unwrap(),
        }
    }
}

impl From<ForgeWorkflow> for Workflow {
    fn from(value: ForgeWorkflow) -> Self {
        Self {
            agents: vec![value.title_agent, value.developer_agent],
            variables: Variables::default(),
        }
    }
}
