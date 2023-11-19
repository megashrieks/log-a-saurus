use chrono::{DateTime, Utc};
use serde::Deserialize;

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
pub enum LikeEqual {
    Like(LikeOp),
    Equal(EqualOp)
}

#[derive(Deserialize, Debug)]
pub enum EqualBetween {
    Equal(EqualOp),
    Between(BetweenOp)
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
    pub parent_resource_id: Option<LikeEqual>,
}



pub fn x() {

}
