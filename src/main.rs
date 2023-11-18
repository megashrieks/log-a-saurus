mod log;
mod query_client;
mod log_processor;

use std::sync::Arc;
use tokio::sync::mpsc;
use axum;


pub struct AppState {
    tx: mpsc::Sender<log::LogStructure>,
}

impl AppState {
    fn new(tx: mpsc::Sender<log::LogStructure>) -> AppState {
        return AppState {
            tx
        }
    }
}


#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    let (tx, rx): (mpsc::Sender<log::LogStructure>, mpsc::Receiver<log::LogStructure>) = mpsc::channel(1000);
    let state: AppState = AppState::new(tx);
    let _ = tokio::join!(
        axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(
            log_processor::ingestion_server::router()
            .with_state(Arc::new(state))
            .into_make_service()
        ),
        log_processor::ingestion_worker::log_archiver(rx),
        log_processor::ingestion_worker::log_processor()
    );
}
