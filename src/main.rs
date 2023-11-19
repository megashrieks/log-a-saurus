mod log;
mod query_client;
mod log_processor;
mod config;
mod db;

use std::sync::Arc;
use config::get_env;
use dotenvy::dotenv;
use tokio::sync::mpsc;
use sqlx::postgres::PgPoolOptions;

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
    dotenv().expect(".env file not found");
    let path = "./appendlogs";
    tokio::fs::create_dir_all(&path).await.unwrap();

    let pool = Arc::new(
            PgPoolOptions::new()
            .max_connections(5)
            .connect(&get_env("DB_URL")).await.unwrap()
        );

    let (tx, rx): (mpsc::Sender<log::LogStructure>, mpsc::Receiver<log::LogStructure>) = mpsc::channel(1000000);
    let state: AppState = AppState::new(tx);
    let _ = tokio::join!(
        axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(
            log_processor::ingestion_server::router()
            .with_state(Arc::new(state))
            .into_make_service()
        ),
        log_processor::ingestion_worker::log_archiver(rx),
        log_processor::ingestion_worker::log_processor(pool)
    );
}
