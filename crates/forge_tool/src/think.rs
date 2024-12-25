use std::collections::HashMap;

use anyhow::Result;
use colorize::AnsiColor;
use forge_tool_macros::Description;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Description, ToolTrait};

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct ThoughtData {
    pub thought: String,
    pub next_thought_needed: bool,
    pub thought_number: i32,
    pub total_thoughts: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_revision: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revises_thought: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch_from_thought: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub needs_more_thoughts: Option<bool>,
}

/// This tool helps analyze problems through a flexible thinking process that
/// can adapt and evolve. Each thought can build on, question, or revise
/// previous insights as understanding deepens.
///
/// When to use this tool:
/// - Breaking down complex problems into steps
/// - Planning and design with room for revision
/// - Analysis that might need course correction
/// - Problems where the full scope might not be clear initially
/// - Problems that require a multi-step solution
/// - Tasks that need to maintain context over multiple steps
/// - Situations where irrelevant information needs to be filtered out
///
/// Key features:
/// - You can adjust total_thoughts up or down as you progress
/// - You can question or revise previous thoughts
/// - You can add more thoughts even after reaching what seemed like the end
/// - You can express uncertainty and explore alternative approaches
/// - Not every thought needs to build linearly - you can branch or backtrack
/// - Generates a solution hypothesis
/// - Verifies the hypothesis based on the Chain of Thought steps
/// - Repeats the process until satisfied
/// - Provides a correct answer
///
/// Parameters explained:
/// - thought: Your current thinking step, which can include:
/// * Regular analytical steps
/// * Revisions of previous thoughts
/// * Questions about previous decisions
/// * Realizations about needing more analysis
/// * Changes in approach
/// * Hypothesis generation
/// * Hypothesis verification
/// - next_thought_needed: True if you need more thinking, even if at what
///   seemed like the end
/// - thought_number: Current number in sequence (can go beyond initial total if
///   needed)
/// - total_thoughts: Current estimate of thoughts needed (can be adjusted
///   up/down)
/// - is_revision: A boolean indicating if this thought revises previous
///   thinking
/// - revises_thought: If is_revision is true, which thought number is being
///   reconsidered
/// - branch_from_thought: If branching, which thought number is the branching
///   point
/// - branch_id: Identifier for the current branch (if any)
/// - needs_more_thoughts: If reaching end but realizing more thoughts needed
///
/// You should:
/// 1. Start with an initial estimate of needed thoughts, but be ready to adjust
/// 2. Feel free to question or revise previous thoughts
/// 3. Don't hesitate to add more thoughts if needed, even at the "end"
/// 4. Express uncertainty when present
/// 5. Mark thoughts that revise previous thinking or branch into new paths
/// 6. Ignore information that is irrelevant to the current step
/// 7. Generate a solution hypothesis when appropriate
/// 8. Verify the hypothesis based on the Chain of Thought steps
/// 9. Repeat the process until satisfied with the solution
/// 10. Provide a single, ideally correct answer as the final output
/// 11. Only set next_thought_needed to false when truly done and a satisfactory
///     answer is reached
#[derive(Clone, Default, Description)]
pub struct Think {
    thought_history: Vec<ThoughtData>,
    branches: HashMap<String, Vec<ThoughtData>>,
}

impl Think {
    fn validate_thought_data(&self, input: ThoughtData) -> Result<ThoughtData> {
        if input.thought_number <= 0 {
            return Err(anyhow::anyhow!("Invalid thoughtNumber: must be positive"));
        }
        if input.total_thoughts <= 0 {
            return Err(anyhow::anyhow!("Invalid totalThoughts: must be positive"));
        }

        Ok(ThoughtData {
            thought: input.thought,
            thought_number: input.thought_number,
            total_thoughts: input.total_thoughts,
            next_thought_needed: input.next_thought_needed,
            is_revision: input.is_revision,
            revises_thought: input.revises_thought,
            branch_from_thought: input.branch_from_thought,
            branch_id: input.branch_id,
            needs_more_thoughts: input.needs_more_thoughts,
        })
    }

    fn format_thought(&self, thought_data: &ThoughtData) -> String {
        let (prefix, context) = match (thought_data.is_revision, &thought_data.branch_from_thought)
        {
            (Some(true), _) => (
                "🔄 Revision".yellow().to_string(),
                format!(
                    " (revising thought {})",
                    thought_data.revises_thought.unwrap_or(0)
                ),
            ),
            (_, Some(branch)) => (
                "🌿 Branch".green().to_string(),
                format!(
                    " (from thought {}, ID: {})",
                    branch,
                    thought_data.branch_id.as_ref().unwrap_or(&String::new())
                ),
            ),
            _ => ("💭 Thought".blue().to_string(), String::new()),
        };

        let header = format!(
            "{} {}/{}{}",
            prefix, thought_data.thought_number, thought_data.total_thoughts, context
        );
        let border_len = header.len().max(thought_data.thought.len()) + 4;
        let border = "─".repeat(border_len);

        let thought_data = format!("{:width$}", thought_data.thought, width = border_len - 2);

        format!(
            "\n┌{}┐\n│ {} │\n├{}┤\n│ {} │\n└{}┘",
            border, header, border, thought_data, border
        )
    }

    fn process_thought(&mut self, input: ThoughtData) -> Result<serde_json::Value> {
        let mut thought_data = self.validate_thought_data(input)?;

        if thought_data.thought_number > thought_data.total_thoughts {
            thought_data.total_thoughts = thought_data.thought_number;
        }

        self.thought_history.push(thought_data.clone());

        if let (Some(_), Some(branch_id)) =
            (thought_data.branch_from_thought, &thought_data.branch_id)
        {
            self.branches
                .entry(branch_id.clone())
                .or_default()
                .push(thought_data.clone());
        }

        eprintln!("{}", self.format_thought(&thought_data));

        let result = serde_json::json!({
            "thoughtNumber": thought_data.thought_number,
            "totalThoughts": thought_data.total_thoughts,
            "nextThoughtNeeded": thought_data.next_thought_needed,
            "branches": self.branches.keys().collect::<Vec<_>>(),
            "thoughtHistoryLength": self.thought_history.len()
        });

        Ok(result)
    }
}

#[async_trait::async_trait]
impl ToolTrait for Think {
    type Input = ThoughtData;
    type Output = serde_json::Value;

    async fn call(&self, input: Self::Input) -> Result<Self::Output, String> {
        let mut thinker = self.clone();
        let thought_result = thinker.process_thought(input).map_err(|e| e.to_string())?;
        Ok(thought_result)
    }
}
