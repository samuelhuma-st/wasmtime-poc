use models::WorkflowData;
use utils::collect_wasm_files;
use workflow_service::WorkflowService;

use std::sync::{Arc, Mutex};

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

mod models;
mod utils;
mod workflow_runner;
mod workflow_service;

#[derive(Debug)]
struct AppState {
    nodes: Mutex<Vec<(String, String)>>,
}

#[get("/")]
async fn index(_data: web::Data<AppState>, req_body: web::Json<WorkflowData>) -> impl Responder {
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

    let all_nodes = collect_wasm_files("/home/hm-samuel/projects/projets-test/wasmtime-poc/target/wasm32-wasip1/debug");
    
    let app_data = Arc::new(AppState {
        nodes: Mutex::new(all_nodes),
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
