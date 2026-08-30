use serde::{Deserialize, Serialize};

/// Returned by GET /api/registry/funds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryFund {
    pub id: String,
    pub name: String,
    pub description: String,
    pub venmo_username: String,
    pub total_usd: f64,
}

/// Submitted by the frontend to POST /api/registry/contribute.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionRequest {
    pub fund_id: String,
    pub contributor_name: String,
    pub amount_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionResponse {
    pub success: bool,
    pub fund_id: String,
    pub new_total_usd: f64,
    pub venmo_username: String,
}
