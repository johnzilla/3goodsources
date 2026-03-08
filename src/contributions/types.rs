use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The current status of a community proposal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatus {
    Pending,
    Approved,
    Rejected,
    Withdrawn,
}

/// The type of action a proposal requests.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalAction {
    AddSource,
    UpdateSource,
    RemoveSource,
    AddCategory,
    UpdateCategory,
}

/// Whether a vote supports or opposes a proposal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoteChoice {
    Support,
    Oppose,
}

/// A single vote on a proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter: String,
    pub vote: VoteChoice,
    pub timestamp: DateTime<Utc>,
}

/// A community proposal. The id is NOT stored in the struct -- it is the HashMap key.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Proposal {
    pub action: ProposalAction,
    pub status: ProposalStatus,
    pub category: String,
    pub proposer: String,
    pub created_at: DateTime<Utc>,
    pub data: serde_json::Value,
    pub votes: Vec<Vote>,
}

impl Default for Proposal {
    fn default() -> Self {
        Self {
            action: ProposalAction::AddSource,
            status: ProposalStatus::Pending,
            category: String::new(),
            proposer: String::new(),
            created_at: Utc::now(),
            data: serde_json::Value::Null,
            votes: Vec::new(),
        }
    }
}

/// Summary view of a proposal for list endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalSummary {
    pub id: Uuid,
    pub action: ProposalAction,
    pub status: ProposalStatus,
    pub category: String,
    pub proposer: String,
    pub created_at: DateTime<Utc>,
}

/// Query parameters for filtering proposals.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ProposalFilterParams {
    pub status: Option<String>,
    pub category: Option<String>,
}
