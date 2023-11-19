use std::sync::Arc;

use axum::{extract::State, response::Json};
use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::{AppState, log, db::query_logs};

#[derive(Deserialize, Debug)]
pub struct LikeOp {
    pub like: String
}

#[derive(Deserialize, Debug)]
pub struct EqualOp {
    pub equals: String
}

#[derive(Deserialize, Debug)]
pub struct BetweenOp {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum LikeEqual {
    Like(LikeOp),
    Equal(EqualOp)
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum EqualBetween {
    Equal(EqualOp),
    Between(BetweenOp)
}

#[derive(Deserialize, Debug)]
pub struct Pagination {
    pub top: u32,
    pub offset: u32
}

#[derive(Deserialize, Debug)]
pub struct LogMetadata {
    pub parent_resource_id: Option<LikeEqual>
}

#[derive(Deserialize, Debug)]
pub struct Query {
    pub level: Option<LikeEqual>,
    pub message: Option<LikeEqual>,
    pub resource_id: Option<LikeEqual>,
    pub timestamp: Option<EqualBetween>,
    pub trace_id: Option<LikeEqual>,
    pub span_id: Option<LikeEqual>,
    pub commit: Option<LikeEqual>,

    #[serde(flatten)]
    pub metadata: Option<LogMetadata>,
    pub pagination: Pagination
}


pub async fn query_resolver(
    State(state): State<Arc<AppState>>, Json(query): Json<Query>
) -> Json<Vec<log::LogStructure>> {
    let pool = &state.clone().pool;
    Json(query_logs(pool, query).await)
}

