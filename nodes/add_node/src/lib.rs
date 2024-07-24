#[allow(warnings)]
mod bindings;

use bindings::Guest;

struct AddNode;

#[derive(Debug, serde::Serialize)]
struct Output {
    result: i32,
    description: String,
}

impl Guest for AddNode {
    fn execute(params: String) -> String {
        if let Ok(value) = serde_json::from_str::<Vec<i32>>(params.as_str()) {
            let data_output = Output {
                result: value.iter().sum(),
                description: String::from("This node performs an addition"),
            };

            let output = serde_json::to_value(&data_output).unwrap().to_string();
            return output;
        } else {
            String::from("Numbers to add not found")
        }
    }
}

bindings::export!(AddNode with_types_in bindings);
