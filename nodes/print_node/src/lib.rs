#[allow(warnings)]
mod bindings;

use bindings::Guest;

struct PrintNode;

#[derive(Debug, serde::Serialize)]
struct Output {
    description: String,
}

impl Guest for PrintNode {
    fn execute(params: String) -> String {
        let output_data = Output {
            description: String::from("This node displays a response in your terminal"),
        };

        let output = serde_json::to_value(&output_data).unwrap().to_string();
        return output;
    }
}

bindings::export!(PrintNode with_types_in bindings);
