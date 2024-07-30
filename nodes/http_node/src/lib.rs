#[allow(warnings)]
mod bindings;
mod custom_http;

use std::fmt;

use bindings::{wasi::http::types::Method, Guest};
use custom_http::send_http_get_request;
use serde::{Deserialize, Serialize, Serializer};
use serde::de::{self, Deserializer, Visitor};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
struct QueryParameter {
    key: String,
    value: String,
}

impl<'de> Deserialize<'de> for Method {
    fn deserialize<D>(deserializer: D) -> Result<Method, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MethodVisitor;

        impl<'de> Visitor<'de> for MethodVisitor {
            type Value = Method;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid HTTP method")
            }

            fn visit_str<E>(self, value: &str) -> Result<Method, E>
            where
                E: de::Error,
            {
                match value.to_uppercase().as_str() {
                    "GET" => Ok(Method::Get),
                    "HEAD" => Ok(Method::Head),
                    "POST" => Ok(Method::Post),
                    "PUT" => Ok(Method::Put),
                    "DELETE" => Ok(Method::Delete),
                    "CONNECT" => Ok(Method::Connect),
                    "OPTIONS" => Ok(Method::Options),
                    "TRACE" => Ok(Method::Trace),
                    "PATCH" => Ok(Method::Patch),
                    other => Ok(Method::Other(other.to_string())),
                }
            }
        }

        deserializer.deserialize_str(MethodVisitor)
    }
}
impl Serialize for Method {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Method::Get => serializer.serialize_str("GET"),
            Method::Head => serializer.serialize_str("HEAD"),
            Method::Post => serializer.serialize_str("POST"),
            Method::Put => serializer.serialize_str("PUT"),
            Method::Delete => serializer.serialize_str("DELETE"),
            Method::Connect => serializer.serialize_str("CONNECT"),
            Method::Options => serializer.serialize_str("OPTIONS"),
            Method::Trace => serializer.serialize_str("TRACE"),
            Method::Patch => serializer.serialize_str("PATCH"),
            Method::Other(other) => serializer.serialize_str(other),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InputData {
    method: Method,
    url: String,
    headers: Option<Vec<QueryParameter>>,
    body: Option<Value>,
    connect_timeout: Option<u64>,
    first_by_timeout: Option<u64>,
    between_bytes_timeout: Option<u64>,
}

struct Component;

#[derive(Debug, serde::Serialize)]
struct Output {
    data: Option<String>,
    error: Option<String>,
}

impl Guest for Component {
    fn execute(params: String) -> String {
        let input_data = serde_json::from_str::<InputData>(&params);
        if let Ok(data) = input_data {
            match send_http_get_request(data) {
                Ok(response) => {
                    println!("HTTP Response Status: {}", response.status);
                    let output_data = Output {
                        data: Some(String::from_utf8_lossy(&response.body).to_string()),
                        error: None,
                    };

                    return json_to_string(output_data);
                }
                Err(e) => {
                    let output_data = Output {
                        data: None,
                        error: Some(format!("Error sending HTTP request: {}", e)),
                    };

                    return json_to_string(output_data);
                }
            }
        } else {
            let output_data = Output {
                data: None,
                error: Some("Cannot get output data".to_string()),
            };

            return json_to_string(output_data);
        }
    }
}

fn json_to_string(output_data: Output) -> String {
    let output = serde_json::to_value(&output_data).unwrap().to_string();
    output
}

bindings::export!(Component with_types_in bindings);
