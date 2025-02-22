use serde::{Deserialize, Serialize};

use crate::{Agent, AgentId};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub agents: Vec<Agent>,
}

impl Workflow {
    fn find_agent(&self, id: &AgentId) -> Option<&Agent> {
        self.agents
            .iter()
            .filter(|a| a.enable)
            .find(|a| a.id == *id)
    }

    pub fn get_agent(&self, id: &AgentId) -> crate::Result<&Agent> {
        self.find_agent(id)
            .ok_or_else(|| crate::Error::AgentUndefined(id.clone()))
    }
}
