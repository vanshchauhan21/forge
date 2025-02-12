use forge_domain::{
    Agent, AgentBuilder, AgentId, DispatchEvent, Environment, NamedTool, Prompt, Workflow,
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
        let agent = AgentBuilder::default().subscribe(vec![DispatchEvent::USER_TASK.to_string()]);

        let title_agent = agent
            .clone()
            .id(AgentId::new("title"))
            .model(env.small_model_id.clone())
            .user_prompt(Prompt::new(
                "<technical_content>{{event.value}}</technical_content>",
            ))
            .system_prompt(Prompt::new(TITLE_GENERATOR_TEMPLATE))
            .max_turns(1u64)
            .tools(vec![DispatchEvent::tool_name()]);

        let developer_agent = agent
            .clone()
            .id(AgentId::new("developer"))
            .model(env.large_model_id.clone())
            .ephemeral(false)
            .user_prompt(Prompt::new("<task>{{event.value}}</task>"))
            .system_prompt(Prompt::new(SOFTWARE_ENGINEER_TEMPLATE))
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
            events: Default::default(),
        }
    }
}
