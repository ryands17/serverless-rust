use aws_sdk_dynamodb::{types::AttributeValue, Client as DynamoDbClient};
use bon::Builder;
use lambda_http::{
    http::StatusCode, run, service_fn, tracing, Error as LambdaError, IntoResponse, Request,
    RequestPayloadExt,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use simple_fn::utils::response;
use ulid::Ulid;

#[derive(Debug, Serialize, Deserialize, Builder)]
struct Person {
    id: String,
    first_name: String,
    last_name: String,
    age: u32,
}

#[derive(Debug, Builder)]
struct Lambda {
    dynamodb_client: DynamoDbClient,
    table_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LambdaInput {
    first_name: String,
    last_name: String,
    age: u32,
}

impl From<LambdaInput> for Person {
    fn from(value: LambdaInput) -> Self {
        Person::builder()
            .id(Ulid::new().to_string())
            .first_name(value.first_name)
            .last_name(value.last_name)
            .age(value.age)
            .build()
    }
}

impl Lambda {
    pub async fn handler(&self, event: Request) -> Result<impl IntoResponse, LambdaError> {
        let body: Option<LambdaInput> = match event.payload() {
            Ok(p) => p,
            Err(e) => {
                return Ok(response::api_response(
                    StatusCode::BAD_REQUEST,
                    e.to_string(),
                ));
            }
        };

        let input = match body {
            Some(p) => p,
            None => {
                return Ok(response::api_response(
                    StatusCode::BAD_REQUEST,
                    "Invalid payload",
                ));
            }
        };

        let person: Person = input.into();

        let put_item_request = self
            .dynamodb_client
            .put_item()
            .table_name(self.table_name.clone())
            .item("firstName", AttributeValue::S(person.first_name.clone()))
            .item("lastName", AttributeValue::S(person.last_name.clone()))
            .item("age", AttributeValue::N(person.age.to_string()))
            .send()
            .await;

        match put_item_request {
            Ok(_) => {
                let resp = response::api_response(StatusCode::OK, json!({ "person": person }));
                Ok(resp)
            }
            Err(_) => {
                let resp = response::api_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error storing person info",
                );
                Ok(resp)
            }
        }
    }
}

/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    tracing::init_default_subscriber();

    let config = aws_config::load_from_env().await;
    let dynamodb_client = DynamoDbClient::new(&config);

    let table_name = std::env::var("TABLE_NAME").expect("TABLE_NAME env var should be set");

    let lambda = Lambda::builder()
        .dynamodb_client(dynamodb_client)
        .table_name(table_name)
        .build();

    run(service_fn(|event: Request| lambda.handler(event))).await
}
