mod log;
mod query_client;
mod log_processor;
mod config;
mod db;

use std::sync::Arc;
use config::get_env;
use dotenvy::dotenv;
use log_processor::ingestion_server::create_new_log;
use query_client::query_resolver;
use tokio::sync::mpsc;
use sqlx::{postgres::PgPoolOptions, Postgres, Pool};

use axum::{self, Router, routing::post};


pub struct AppState {
    tx: mpsc::Sender<log::LogStructure>,
    pool: Arc<Pool<Postgres>>
}

impl AppState {
    fn new(tx: mpsc::Sender<log::LogStructure>, pool: Arc<Pool<Postgres>>) -> AppState {
        return AppState {
            tx, pool
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
    let state: Arc<AppState> = Arc::new(AppState::new(tx, pool.clone()));

    let app = Router::new()
        .route("/", post(create_new_log))
        .route("/query", post(query_resolver))
        .with_state(state);

    let _ = tokio::join!(
        axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(
            app.into_make_service()
        ),
        log_processor::ingestion_worker::log_archiver(rx),
        log_processor::ingestion_worker::log_processor(pool)
    );
}
