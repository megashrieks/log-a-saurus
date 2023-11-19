use std::sync::Arc;

use axum::extract::{ State, Json };

use crate::{AppState, log};

pub async fn create_new_log(State(state): State<Arc<AppState>>, Json(body): Json<log::LogStructure>) {
    let tx = &state.clone().tx;
    println!("received: {:?}", body);
    tx.send(body).await.unwrap();
}
