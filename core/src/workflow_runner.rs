use std::collections::HashMap;

use component::ResourceTable;
use serde_json::Value;
use wasmtime::component::{Instance, Linker, Val};
use wasmtime::*;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView};

use crate::utils::resolve_references;
use crate::{
    models::WorkflowData,
    utils::{build_dependency_graph, topological_sort},
};
pub struct WorkflowRunner {}

// wasmtime config
struct MyState {
    ctx: WasiCtx,
    table: ResourceTable,
}

impl WasiView for MyState {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
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

        let trigger_node = workflow_data
            .nodes
            .iter()
            .find(|&n| n.node_type == "trigger");

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
                        &execution_results,
                    );
                    println!("input_data = {resolved_params:?}");

                    let value = match resolved_params.get("value") {
                        Some(x) => x,
                        None => &"".to_string(),
                    };

                    let mut result = String::new();

                    // execute
                    let module_wasm = std::fs::read(node_box.1.clone()).unwrap();
                    // let module = Module::new(&engine, &module_wasm).unwrap();
                    let module =
                        wasmtime::component::Component::new(&engine, &module_wasm).unwrap();

                    // Create a WASI context
                    // let wasi = WasiCtxBuilder::new().inherit_stdio().build();
                    let mut builder = WasiCtxBuilder::new();
                    let mut store = wasmtime::Store::new(
                        &engine,
                        MyState {
                            ctx: builder.build(),
                            table: ResourceTable::new(),
                        },
                    );

                    // Create a linker and add WASI to the imports
                    let mut linker = Linker::<MyState>::new(&engine);
                    wasmtime_wasi::add_to_linker_sync(&mut linker).unwrap();

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
                        let output_data: Value = serde_json::from_str(a_str).unwrap();
                        execution_results.insert(
                            current_node.name.clone(),
                            serde_json::from_str(format!("{{\"json\": {output_data}}}").as_str())
                                .unwrap(),
                        );
                    }

     
                    
                    // let output_data: Value = serde_json::from_str(a).unwrap();

                    // execution_results.insert(
                    //     current_node.name.clone(),
                    //     serde_json::from_str(format!("{{\"json\": {output_data}}}").as_str())
                    //         .unwrap(),
                    // );

                    println!("Result from hello_world: {:?}", res);

                    // let wasi = WasiCtxBuilder::new().inherit_stdio().build();
                    // let mut store = Store::new(&engine, wasi);

                    // let mut linker = Linker::new(&engine);
                    // wasmtime_wasi::add_to_linker(&mut linker, |s| s).unwrap();

                    // // Créer une instance du module avec les imports WASI
                    // let instance = linker.instantiate(&mut store, &module).unwrap();

                    // //  Allouer de la mémoire pour la chaîne de caractères dans le module WebAssembly
                    // let memory = instance
                    //     .get_memory(&mut store, "memory")
                    //     .expect("Memory not found");

                    // let input_len = value.len();
                    // let ptr = 0x10000; // Adresse arbitraire dans la mémoire WebAssembly

                    // // Copier la chaîne de caractères dans la mémoire WebAssembly
                    // memory
                    //     .write(&mut store, ptr, value.as_bytes())
                    //     .expect("Failed to write to memory");
                    // println!("memo {:?}", memory);
                    // // Appeler la fonction exportée `add`
                    // let add = instance
                    //     .get_typed_func::<(i32, i32), i32>(&mut store, "execute")
                    //     .unwrap();
                    // let result_ptr = add
                    //     .call(&mut store, (ptr as i32, input_len as i32))
                    //     .unwrap();

                    // // Read the result string from WebAssembly memory
                    // let mut buf = Vec::new();
                    // for i in 0..100 {
                    //     // Read up to 100 bytes, assuming the output won't be longer
                    //     let byte = memory.data(&store)[result_ptr as usize + i];
                    //     if byte == 0 {
                    //         break;
                    //     } // Null terminator
                    //     buf.push(byte);
                    // }
                    // let result_str = String::from_utf8(buf).expect("Invalid UTF-8");

                    // println!("Result: {}", result_str);

                    // let result_format = result.0.to_string();

                    // let output_data: Value = serde_json::from_str(&result_format).unwrap();

                    // execution_results.insert(
                    //     current_node.name.clone(),
                    //     serde_json::from_str(format!("{{\"json\": {output_data}}}").as_str())
                    //         .unwrap(),
                    // );

                    println!("execution_results = {execution_results:?}");
                } else {
                    println!("Node type {} not found in all_nodes", current_node.name);
                }
            }
        }
    }
}
