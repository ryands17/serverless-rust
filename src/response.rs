use lambda_http::{http::StatusCode, Response};

pub fn api_response(status_code: StatusCode, body: serde_json::Value) -> Response<String> {
    Response::builder()
        .status(status_code)
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .unwrap()
}
