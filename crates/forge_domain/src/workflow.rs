use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use forge_stream::MpscStream;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{
    Agent, AgentId, App, ChatRequest, ChatResponse, Context, DispatchEvent, Orchestrator,
    SystemContext,
};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub agents: Vec<Agent>,
    #[serde(skip)]
    pub events: HashMap<String, DispatchEvent>,
}

impl Workflow {
    pub fn find_agent_mut(&mut self, id: &AgentId) -> Option<&mut Agent> {
        self.agents.iter_mut().find(|a| a.id == *id)
    }

    pub fn find_agent(&self, id: &AgentId) -> Option<&Agent> {
        self.agents.iter().find(|a| a.id == *id)
    }

    pub fn get_agent_mut(&mut self, id: &AgentId) -> crate::Result<&mut Agent> {
        self.find_agent_mut(id)
            .ok_or_else(|| crate::Error::AgentUndefined(id.clone()))
    }

    pub fn get_agent(&self, id: &AgentId) -> crate::Result<&Agent> {
        self.find_agent(id)
            .ok_or_else(|| crate::Error::AgentUndefined(id.clone()))
    }

    pub fn entries(&self, event_name: &str) -> Vec<Agent> {
        self.agents
            .iter()
            .filter(|a| a.state.turn_count < a.max_turns)
            .filter(|a| a.subscribe.contains(&event_name.to_string()))
            .cloned()
            .collect::<Vec<_>>()
    }
}

#[derive(Default, Clone)]
pub struct ConcurrentWorkflow {
    workflow: Arc<RwLock<Workflow>>,
}

impl ConcurrentWorkflow {
    pub fn new(workflow: Workflow) -> Self {
        Self { workflow: Arc::new(RwLock::new(workflow)) }
    }

    pub async fn context(&self, id: &AgentId) -> Option<Context> {
        let guard = self.workflow.read().await;
        guard.find_agent(id).and_then(|a| a.state.context.clone())
    }

    pub async fn insert_event(&self, event: DispatchEvent) {
        let mut guard = self.workflow.write().await;
        guard.events.insert(event.name.to_string(), event);
    }

    pub async fn get_event(&self, name: &str) -> Option<DispatchEvent> {
        let guard = self.workflow.read().await;
        guard.events.get(name).cloned()
    }

    pub async fn find_agent(&self, id: &AgentId) -> Option<Agent> {
        let guard = self.workflow.read().await;
        guard.find_agent(id).cloned()
    }

    pub async fn get_agent(&self, agent: &AgentId) -> crate::Result<Agent> {
        let guard = self.workflow.read().await;
        guard.get_agent(agent).cloned()
    }

    pub async fn set_context(&self, agent: &AgentId, context: Context) -> crate::Result<()> {
        let mut guard = self.workflow.write().await;
        guard.get_agent_mut(agent)?.state.context = Some(context);
        Ok(())
    }

    pub async fn entries(&self, event_name: &str) -> Vec<Agent> {
        let guard = self.workflow.read().await;
        guard.entries(event_name)
    }

    pub async fn complete_turn(&self, agent: &AgentId) -> crate::Result<()> {
        let mut guard = self.workflow.write().await;
        let agent = guard.get_agent_mut(agent)?;
        let max_turns = agent.max_turns;
        if agent.state.turn_count >= max_turns {
            return Err(crate::Error::MaxTurnsReached(agent.id.clone(), max_turns));
        } else {
            agent.state.turn_count += 1;
        }

        Ok(())
    }

    pub fn execute<'a, F: App + 'a>(
        &'a self,
        domain: Arc<F>,
        request: ChatRequest,
        ctx: SystemContext,
    ) -> MpscStream<anyhow::Result<crate::AgentMessage<ChatResponse>>> {
        let workflow = self.clone();

        MpscStream::spawn(move |tx| async move {
            let tx = Arc::new(tx);
            let orch = Orchestrator::new(domain, workflow, ctx, Some(tx.clone()));
            match orch.execute(request).await {
                Ok(_) => {}
                Err(err) => tx.send(Err(err)).await.unwrap(),
            }
        })
    }

    /// Initialize the concurrent workflow with the given workflow. If None is
    /// provided then it's initialized to an empty workflow.
    pub async fn init(&self, workflow: Option<Workflow>) {
        let mut guard = self.workflow.write().await;
        if let Some(workflow) = workflow {
            *guard = workflow;
        } else {
            for agent in guard.agents.iter_mut() {
                agent.state = Default::default();
            }
        }
    }
}

impl FromStr for Workflow {
    type Err = toml::de::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::de::from_str(s)
    }
}
