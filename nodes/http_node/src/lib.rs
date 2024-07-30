#[allow(warnings)]
mod bindings;
pub mod custom_http;

use custom_http::send_http_get_request;
use serde_json::Value;
use serde::{ Deserialize, Serialize };

use crate::bindings::Guest;
use bindings::wasi::http::types::Method as WasiMethod;

#[derive(Debug, Serialize, Deserialize)]
struct QueryParameter {
    key: String,
    value: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InputData {
    method: MethodString,
    url: String,
    headers: Option<Vec<QueryParameter>>,
    body: Option<Value>,
    connect_timeout: Option<u64>,
    first_by_timeout: Option<u64>,
    between_bytes_timeout: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
struct MethodString {
    method: String,
}
// Implémentation From pour une conversion facile
impl From<WasiMethod> for MethodString {
    fn from(method: WasiMethod) -> Self {
        let method_str = match method {
            WasiMethod::Get => "Get".to_string(),
            WasiMethod::Post => "Post".to_string(),
            WasiMethod::Head => "Head".to_string(),
            WasiMethod::Put => "Put".to_string(),
            WasiMethod::Delete => "Delete".to_string(),
            WasiMethod::Connect => "Connect".to_string(),
            WasiMethod::Options => "Options".to_string(),
            WasiMethod::Trace => "Trace".to_string(),
            WasiMethod::Patch => "Patch".to_string(),
            WasiMethod::Other(_) => "Other".to_string(),
        };
        MethodString { method: method_str }
    }
}
// Implémentation Into pour une conversion facile
impl Into<WasiMethod> for MethodString {
    fn into(self) -> WasiMethod {
        match self.method.as_str() {
            "Get" => WasiMethod::Get,
            "Post" => WasiMethod::Post,
            "Head" => WasiMethod::Head,
            "Put" => WasiMethod::Put,
            "Delete" => WasiMethod::Delete,
            "Connect" => WasiMethod::Connect,
            "Options" => WasiMethod::Options,
            "Trace" => WasiMethod::Trace,
            "Patch" => WasiMethod::Patch,
            _ => WasiMethod::Other(String::new()),
        }
    }
}

struct HttpRequest;

#[derive(Debug, serde::Serialize)]
struct Output {
    data: Option<String>,
    error: Option<String>,
}

impl Guest for HttpRequest {
    fn execute(params: String) -> String {
        let input_data = serde_json::from_str::<InputData>(&params);
        println!("started");
        if let Ok(data) = input_data {
            match send_http_get_request(data) {
                Ok(response) => {
                    let output_data = Output {
                        data: Some(String::from_utf8_lossy(&response.body).to_string()),
                        error: None,
                    };

                    let output = serde_json::to_value(&output_data).unwrap().to_string();
                    return output;
                }
                Err(e) => {
                    let output_data = Output {
                        data: None,
                        error: Some(format!("Error sending HTTP request: {}", e)),
                    };
                    let output = serde_json::to_value(&output_data).unwrap().to_string();
                    return output;
                }
            }
        } else {
            let output_data = Output {
                data: None,
                error: Some(String::from("Cannot get input_data")),
            };
            let output = serde_json::to_value(&output_data).unwrap().to_string();
            return output;
        }
    }
}

bindings::export!(HttpRequest with_types_in bindings);
