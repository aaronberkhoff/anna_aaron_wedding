use crate::{error::AppError, state::AppState};
use axum::{extract::State, Json};
use shared::models::registry::{ContributionRequest, ContributionResponse, RegistryFund};
use uuid::Uuid;

pub async fn list_funds(
    State(state): State<AppState>,
) -> Result<Json<Vec<RegistryFund>>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT f.id as "id!", f.name as "name!", f.description as "description!",
                  f.venmo_username as "venmo_username!",
                  COALESCE(SUM(c.amount_usd), 0.0) as "total_usd!: f64"
           FROM registry_funds f
           LEFT JOIN fund_contributions c ON c.fund_id = f.id
           GROUP BY f.id
           ORDER BY f.name"#
    )
    .fetch_all(&state.pool)
    .await?;

    let funds = rows
        .into_iter()
        .map(|row| RegistryFund {
            id: row.id,
            name: row.name,
            description: row.description,
            venmo_username: row.venmo_username,
            total_usd: row.total_usd,
        })
        .collect();

    Ok(Json(funds))
}

pub async fn contribute(
    State(state): State<AppState>,
    Json(payload): Json<ContributionRequest>,
) -> Result<Json<ContributionResponse>, AppError> {
    if payload.amount_usd <= 0.0 {
        return Err(AppError::Validation(
            "amount must be greater than zero".to_string(),
        ));
    }
    if payload.contributor_name.trim().is_empty() {
        return Err(AppError::Validation("name is required".to_string()));
    }

    let fund = sqlx::query!(
        "SELECT venmo_username FROM registry_funds WHERE id = ?",
        payload.fund_id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let contribution_id = Uuid::new_v4().to_string();
    let contributor_name = payload.contributor_name.trim();
    sqlx::query!(
        "INSERT INTO fund_contributions (id, fund_id, contributor_name, amount_usd)
         VALUES (?, ?, ?, ?)",
        contribution_id,
        payload.fund_id,
        contributor_name,
        payload.amount_usd
    )
    .execute(&state.pool)
    .await?;

    let total = sqlx::query!(
        r#"SELECT COALESCE(SUM(amount_usd), 0.0) as "total!: f64"
           FROM fund_contributions WHERE fund_id = ?"#,
        payload.fund_id
    )
    .fetch_one(&state.pool)
    .await?
    .total;

    Ok(Json(ContributionResponse {
        success: true,
        fund_id: payload.fund_id,
        new_total_usd: total,
        venmo_username: fund.venmo_username,
    }))
}
