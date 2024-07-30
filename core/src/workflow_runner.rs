use std::collections::HashMap;

use component::ResourceTable;
use serde_json::Value;
use wasmtime::component::{ Instance, Linker, Val };
use wasmtime::*;
use wasmtime_wasi::{ WasiCtx, WasiCtxBuilder, WasiView };
use crate::utils::resolve_references;
use crate::{ models::WorkflowData, utils::{ build_dependency_graph, topological_sort } };
use wasmtime_wasi_http::WasiHttpCtx;
use wasmtime_wasi_http::WasiHttpView;

pub struct WorkflowRunner {}
struct MyState {
    ctx: WasiCtx,
    table: ResourceTable,
    http: WasiHttpCtx,
}

impl WasiView for MyState {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

impl WasiHttpView for MyState {
    fn ctx(&mut self) -> &mut WasiHttpCtx {
        &mut self.http
    }
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

impl WorkflowRunner {
    pub fn run(workflow_data: &WorkflowData, all_nodes: Vec<(String, String)>) {
        let mut execution_results: HashMap<String, Value> = HashMap::new();

        let graph = build_dependency_graph(workflow_data);
        let sorted_nodes = topological_sort(&workflow_data, &graph);

        let trigger_node = workflow_data.nodes.iter().find(|&n| n.node_type == "trigger");

        if let None = trigger_node {
            println!("Aucun nœud de type 'trigger' trouvé.");

            return;
        }

        if sorted_nodes.is_empty() {
            println!("L'ordre des nœuds est incorrect.");

            return;
        }

        // Vérifier si le nœud de départ existe
        if !sorted_nodes.contains(&trigger_node.unwrap().id) {
            println!("Le nœud de départ n'existe pas dans l'ordre trié.");

            return;
        }

        // Exécution des nœuds dans l'ordre topologique à partir du nœud de départ
        let start_index = sorted_nodes
            .iter()
            .position(|id| *id == trigger_node.unwrap().id)
            .unwrap();

        let engine = Engine::default();

        for node_id in &sorted_nodes[start_index..] {
            if let Some(current_node) = workflow_data.nodes.iter().find(|&n| n.id == *node_id) {
                let a = current_node.node_type.as_str();
                if let Some(node_box) = all_nodes.iter().find(|x| x.0 == a) {
                    let resolved_params = resolve_references(
                        &current_node.parameters.clone().unwrap(),
                        &execution_results
                    );
                    println!("input_data = {resolved_params:?}");

                    let value = match resolved_params.get("value") {
                        Some(x) => x,
                        None => &"".to_string(),
                    };

                    // execute
                    let module_wasm = std::fs::read(node_box.1.clone()).unwrap();
                    let module = wasmtime::component::Component
                        ::new(&engine, &module_wasm)
                        .unwrap();

                    // Create a WASI context
                    let mut builder = WasiCtxBuilder::new();
                    let mut store = wasmtime::Store::new(&engine, MyState {
                        ctx: builder.build(),
                        table: ResourceTable::new(),
                        http: WasiHttpCtx::new(),
                    });

                    // Create a linker and add WASI to the imports
                    let mut linker = Linker::<MyState>::new(&engine);
                    wasmtime_wasi::add_to_linker_sync(&mut linker).unwrap();
                    wasmtime_wasi_http::add_only_http_to_linker_async(&mut linker).unwrap();

                    // Instantiate the module with the WASI imports
                    let instance: Instance = linker.instantiate(&mut store, &module).unwrap();

                    let execute_func = instance.get_func(&mut store, "execute").unwrap();

                    let mut res = [Val::String("".into())];
                    if node_box.0 == "trigger".to_string() {
                        execute_func.call(&mut store, &[], &mut res).unwrap();
                    } else {
                        execute_func
                            .call(&mut store, &[Val::String(value.to_string())], &mut res)
                            .unwrap();
                    }

                    let a = res.first().unwrap();
                    if let Val::String(s) = a {
                        let a_str: &str = &s;
                        println!("as_str 👍: {:?}", a_str);
                        let output_data: Value = serde_json::from_str(a_str).unwrap();
                        execution_results.insert(
                            current_node.name.clone(),
                            serde_json
                                ::from_str(format!("{{\"json\": {output_data}}}").as_str())
                                .unwrap()
                        );
                    }

                    println!("execution_results = {execution_results:?}");
                } else {
                    println!("Node type {} not found in all_nodes", current_node.node_type);
                }
            }
        }
    }
}
