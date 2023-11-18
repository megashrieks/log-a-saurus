use std::sync::Arc;

use axum::{
    routing::post,
    Router,
    extract:: { State, Json }
};

use crate::{AppState, log};


pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", post(create_new_log))
}

async fn create_new_log(State(state): State<Arc<AppState>>, Json(body): Json<log::LogStructure>) {
    let tx = &state.clone().tx;

    tx.send(body).await.unwrap();
}
