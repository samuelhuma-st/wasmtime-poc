#[allow(warnings)]
mod bindings;

use std::{thread, time::Duration};

use bindings::Guest;
use serde::{Deserialize, Serialize};

struct Component;

#[derive(Debug, Serialize, Deserialize)]
struct Output {
    data: Option<String>,
    error: Option<String>,
}

impl Guest for Component {
    fn execute(params: String) -> String {
        let input_data = serde_json::from_str::<u64>(&params);

        if let Ok(data) = input_data {
            thread::sleep(Duration::from_secs(data));

            let output_data = Output {
                data: Some("Done".to_string()),
                error: None,
            };
            let output = serde_json::to_value(&output_data).unwrap().to_string();
            output
        } else {
            let output_data = Output {
                data: None,
                error: Some("Cannot get duration value".to_string()),
            };

            let output = serde_json::to_value(&output_data).unwrap().to_string();
            output
        }
    }
}

bindings::export!(Component with_types_in bindings);
