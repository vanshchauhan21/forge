use std::collections::HashSet;
use std::sync::Arc;

use async_recursion::async_recursion;
use derive_setters::Setters;
use futures::future::join_all;
use futures::{Stream, StreamExt};
use serde_json::Value;
use tokio::sync::Mutex;

use crate::*;

pub struct AgentMessage<T> {
    pub agent: AgentId,
    pub message: T,
}

#[derive(Setters)]
pub struct Orchestrator {
    provider_svc: Arc<dyn ProviderService>,
    tool_svc: Arc<dyn ToolService>,
    workflow: Arc<Mutex<Workflow>>,
    system_context: SystemContext,
    sender: Option<tokio::sync::mpsc::Sender<AgentMessage<ChatResponse>>>,
}

struct ChatCompletionResult {
    pub content: String,
    pub tool_calls: Vec<ToolCallFull>,
}

impl Orchestrator {
    pub fn new(provider: Arc<dyn ProviderService>, tool: Arc<dyn ToolService>) -> Self {
        Self {
            provider_svc: provider,
            tool_svc: tool,
            workflow: Arc::new(Mutex::new(Workflow::default())),
            system_context: SystemContext::default(),
            sender: None,
        }
    }

    pub async fn agent_context(&self, id: &AgentId) -> Option<Context> {
        let guard = self.workflow.lock().await;
        guard.state.get(id).cloned()
    }

    async fn send_message(&self, agent_id: &AgentId, message: ChatResponse) -> anyhow::Result<()> {
        if let Some(sender) = &self.sender {
            sender
                .send(AgentMessage { agent: agent_id.clone(), message })
                .await?
        }
        Ok(())
    }

    async fn send(&self, agent_id: &AgentId, message: ChatResponse) -> anyhow::Result<()> {
        self.send_message(agent_id, message).await
    }

    fn init_default_tool_definitions(&self) -> Vec<ToolDefinition> {
        self.tool_svc.list()
    }

    fn init_tool_definitions(&self, agent: &Agent) -> Vec<ToolDefinition> {
        let allowed = agent.tools.iter().collect::<HashSet<_>>();
        let mut forge_tools = self.init_default_tool_definitions();

        // Adding self to the list of tool definitions
        forge_tools.push(ReadVariable::tool_definition());
        forge_tools.push(WriteVariable::tool_definition());

        forge_tools
            .into_iter()
            .filter(|tool| allowed.contains(&tool.name))
            .collect::<Vec<_>>()
    }

    fn init_agent_context(&self, agent: &Agent, input: &Variables) -> anyhow::Result<Context> {
        let tool_defs = self.init_tool_definitions(agent);

        let tool_usage_prompt = tool_defs.iter().fold("".to_string(), |acc, tool| {
            format!("{}\n{}", acc, tool.usage_prompt())
        });

        let system_message = agent.system_prompt.render(
            &self
                .system_context
                .clone()
                .tool_information(tool_usage_prompt),
        )?;

        let user_message = ContextMessage::user(agent.user_prompt.render(input)?);

        Ok(Context::default()
            .set_first_system_message(system_message)
            .add_message(user_message)
            .extend_tools(tool_defs))
    }

    async fn collect_messages(
        &self,
        agent: &AgentId,
        mut response: impl Stream<Item = std::result::Result<ChatCompletionMessage, anyhow::Error>>
            + std::marker::Unpin,
    ) -> anyhow::Result<ChatCompletionResult> {
        let mut messages = Vec::new();

        while let Some(message) = response.next().await {
            let message = message?;
            messages.push(message.clone());
            if let Some(content) = message.content {
                self.send(agent, ChatResponse::Text(content.as_str().to_string()))
                    .await?;
            }

            if let Some(usage) = message.usage {
                self.send(agent, ChatResponse::Usage(usage)).await?;
            }
        }

        let content = messages
            .iter()
            .flat_map(|m| m.content.iter())
            .map(|content| content.as_str())
            .collect::<Vec<_>>()
            .join("");

        // From Complete (incase streaming is disabled)
        let mut tool_calls: Vec<ToolCallFull> = messages
            .iter()
            .flat_map(|message| message.tool_call.iter())
            .filter_map(|message| message.as_full().cloned())
            .collect::<Vec<_>>();

        // From partial tool calls
        tool_calls.extend(ToolCallFull::try_from_parts(
            &messages
                .iter()
                .filter_map(|message| message.tool_call.first())
                .clone()
                .filter_map(|tool_call| tool_call.as_partial().cloned())
                .collect::<Vec<_>>(),
        )?);

        // From XML
        tool_calls.extend(ToolCallFull::try_from_xml(&content)?);

        Ok(ChatCompletionResult { content, tool_calls })
    }

    async fn write_variable(
        &self,
        tool_call: &ToolCallFull,
        write: WriteVariable,
    ) -> anyhow::Result<ToolResult> {
        let mut guard = self.workflow.lock().await;
        guard.variables.add(write.name.clone(), write.value.clone());
        Ok(ToolResult::from(tool_call.clone())
            .success(format!("Variable {} set to {}", write.name, write.value)))
    }

    async fn read_variable(
        &self,
        tool_call: &ToolCallFull,
        read: ReadVariable,
    ) -> anyhow::Result<ToolResult> {
        let guard = self.workflow.lock().await;
        let output = guard.variables.get(&read.name);
        let result = match output {
            Some(value) => {
                ToolResult::from(tool_call.clone()).success(serde_json::to_string(value)?)
            }
            None => ToolResult::from(tool_call.clone())
                .failure(format!("Variable {} not found", read.name)),
        };
        Ok(result)
    }

    #[async_recursion(?Send)]
    async fn execute_tool(&self, tool_call: &ToolCallFull) -> anyhow::Result<Option<ToolResult>> {
        if let Some(read) = ReadVariable::parse(tool_call) {
            self.read_variable(tool_call, read).await.map(Some)
        } else if let Some(write) = WriteVariable::parse(tool_call) {
            self.write_variable(tool_call, write).await.map(Some)
        } else if let Some(agent) = self
            .workflow
            .lock()
            .await
            .find_agent(&tool_call.name.clone().into())
        {
            let input = Variables::from(tool_call.arguments.clone());
            self.init_agent(&agent.id, &input).await?;
            Ok(None)
        } else {
            Ok(Some(self.tool_svc.call(tool_call.clone()).await))
        }
    }

    #[async_recursion(?Send)]
    async fn execute_transform(
        &self,
        transforms: &[Transform],
        mut context: Context,
    ) -> anyhow::Result<Context> {
        for transform in transforms.iter() {
            match transform {
                Transform::Assistant {
                    agent_id,
                    token_limit,
                    input: input_key,
                    output: output_key,
                } => {
                    let mut summarize = Summarize::new(&mut context, *token_limit);
                    while let Some(mut summary) = summarize.summarize() {
                        let mut input = Variables::default();
                        input.add(input_key, summary.get());

                        self.init_agent(agent_id, &input).await?;

                        let guard = self.workflow.lock().await;

                        let value = guard
                            .variables
                            .get(output_key)
                            .ok_or(Error::UndefinedVariable(output_key.to_string()))?;

                        summary.set(serde_json::to_string(&value)?);
                    }
                }
                Transform::User { agent_id, input: input_key, output: output_key } => {
                    if let Some(ContextMessage::ContentMessage(ContentMessage {
                        role: Role::User,
                        content,
                        ..
                    })) = context.messages.last_mut()
                    {
                        let mut input = Variables::default();
                        input.add(input_key, Value::from(content.clone()));

                        self.init_agent(agent_id, &input).await?;
                        let guard = self.workflow.lock().await;
                        let value = guard
                            .variables
                            .get(output_key)
                            .ok_or(Error::UndefinedVariable(output_key.to_string()))?;

                        let message = serde_json::to_string(&value)?;

                        content.push_str(&format!("\n<{output_key}>\n{message}\n</{output_key}>"));
                    }
                }
                Transform::Tap { agent_id, input: input_key } => {
                    let mut input = Variables::default();
                    input.add(input_key, context.to_text());

                    // NOTE: Tap transformers will not modify the context
                    self.init_agent(agent_id, &input).await?;
                }
            }
        }

        Ok(context)
    }

    async fn init_agent(&self, agent: &AgentId, input: &Variables) -> anyhow::Result<()> {
        let guard = self.workflow.lock().await;
        let agent = guard.get_agent(agent)?;

        let mut context = if agent.ephemeral {
            self.init_agent_context(agent, input)?
        } else {
            self.workflow
                .lock()
                .await
                .state
                .get(&agent.id)
                .cloned()
                .map(Ok)
                .unwrap_or_else(|| self.init_agent_context(agent, input))?
        };

        let content = agent.user_prompt.render(input)?;

        context = context.add_message(ContextMessage::user(content));

        loop {
            context = self.execute_transform(&agent.transforms, context).await?;

            let response = self
                .provider_svc
                .chat(&agent.model, context.clone())
                .await?;
            let ChatCompletionResult { tool_calls, content } =
                self.collect_messages(&agent.id, response).await?;

            let mut tool_results = Vec::new();

            for tool_call in tool_calls.iter() {
                self.send(&agent.id, ChatResponse::ToolCallStart(tool_call.clone()))
                    .await?;
                if let Some(tool_result) = self.execute_tool(tool_call).await? {
                    tool_results.push(tool_result.clone());
                    self.send(&agent.id, ChatResponse::ToolCallEnd(tool_result))
                        .await?;
                }
            }

            context = context
                .add_message(ContextMessage::assistant(content, Some(tool_calls)))
                .add_tool_results(tool_results.clone());

            if !agent.ephemeral {
                self.workflow
                    .lock()
                    .await
                    .state
                    .insert(agent.id.clone(), context.clone());
            }

            if tool_results.is_empty() {
                return Ok(());
            }
        }
    }

    pub async fn execute(&self, input: &Variables) -> anyhow::Result<()> {
        let guard = self.workflow.lock().await;

        join_all(
            guard
                .get_entries()
                .iter()
                .map(|agent| self.init_agent(&agent.id, input)),
        )
        .await
        .into_iter()
        .collect::<anyhow::Result<Vec<()>>>()?;

        Ok(())
    }
}
