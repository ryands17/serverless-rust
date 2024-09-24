use lambda_http::{http::StatusCode, Response};
use serde_json::{json, Value};

fn merge(a: &mut Value, b: Value) {
    match (a, b) {
        (Value::Object(a), Value::Object(b)) => {
            for (k, v) in b {
                merge(a.entry(k).or_insert(Value::Null), v);
            }
        }
        (a, b) => *a = b,
    }
}

pub fn api_response<T: serde::Serialize>(status_code: StatusCode, body: T) -> Response<String> {
    if status_code.is_success() {
        let mut response = json!({
            "success": status_code.is_success(),
            "errors": Value::Null
        });

        merge(&mut response, json!({ "data": body }));

        return Response::builder()
            .status(status_code)
            .header("Content-Type", "application/json")
            .body(response.to_string())
            .unwrap();
    } else {
        let mut response = json!({
            "success": status_code.is_success(),
            "data": Value::Null
        });

        merge(&mut response, json!({ "errors": body }));

        return Response::builder()
            .status(status_code)
            .header("Content-Type", "application/json")
            .body(response.to_string())
            .unwrap();
    }
}
