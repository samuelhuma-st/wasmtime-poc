use models::WorkflowData;
use utils::parse_workflow_data;
use wasmtime::*;
use wasmtime_wasi::WasiCtxBuilder;
use workflow_service::WorkflowService;

use std::{
    fs, sync::{Arc, Mutex}, thread, time::Duration
};

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};

mod models;
mod utils;
mod workflow_runner;
mod workflow_service;

#[derive(Debug)]
struct AppState {
    nodes: Mutex<Vec<(String, String)>>,
}

#[get("/")]
async fn index(data: web::Data<AppState>, req_body: web::Json<WorkflowData>) -> impl Responder {
    HttpResponse::Ok().body(format!("Started of {}", req_body.0.clone().name))
}
#[post("/manual-trigger")]
async fn manual_trigger(
    data: web::Data<AppState>,
    req_body: web::Json<WorkflowData>,
) -> impl Responder {
    let nodes = data.nodes.lock().unwrap();
    WorkflowService::execute_manually(req_body.0.clone(), nodes.to_vec());

    HttpResponse::Ok().message_body(format!("Worklow {} is executed", req_body.0.name))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_data = Arc::new(AppState {
        nodes: Mutex::new(vec![
           ("add".to_string(), "/home/hm-samuel/projects/projets-test/wasmtime-poc/target/wasm32-wasip1/debug/add_node.wasm".to_string()),
           ("print".to_string(), "/home/hm-samuel/projects/projets-test/wasmtime-poc/target/wasm32-wasip1/debug/print_node.wasm".to_string()),
           ("trigger".to_string(), "/home/hm-samuel/projects/projets-test/wasmtime-poc/target/wasm32-wasip1/debug/trigger_node.wasm".to_string())

        ]),
    });
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::from(app_data.clone()))
            .service(manual_trigger)
            .service(index)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
