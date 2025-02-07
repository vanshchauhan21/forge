#![allow(dead_code)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{Agent, AgentId, Context, Variables};

#[derive(Default, Serialize, Deserialize)]
pub struct Workflow {
    pub agents: Vec<Agent>,
    pub state: HashMap<AgentId, Context>,
    pub variables: Variables,
}

impl Workflow {
    pub fn find_agent(&self, id: &AgentId) -> Option<&Agent> {
        self.agents.iter().find(|a| a.id == *id)
    }

    pub fn get_agent(&self, id: &AgentId) -> crate::Result<&Agent> {
        self.find_agent(id)
            .ok_or_else(|| crate::Error::AgentUndefined(id.clone()))
    }

    pub fn get_entries(&self) -> Vec<&Agent> {
        self.agents.iter().filter(|a| a.entry).collect::<Vec<_>>()
    }
}
