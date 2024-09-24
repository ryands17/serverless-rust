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
use validator::Validate;

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

#[derive(Debug, Serialize, Deserialize, Validate)]
struct LambdaInput {
    #[validate(length(min = 1))]
    #[serde(rename = "firstName")]
    first_name: String,
    #[validate(length(min = 1))]
    #[serde(rename = "lastName")]
    last_name: String,
    #[validate(range(min = 1))]
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
        // validate event body
        let body = match event.payload::<LambdaInput>() {
            Ok(Some(p)) => match p.validate() {
                Ok(_) => p,
                Err(e) => return Ok(response::api(StatusCode::BAD_REQUEST, e)),
            },
            Ok(None) => return Ok(response::api(StatusCode::BAD_REQUEST, "Invalid payload")),
            Err(e) => return Ok(response::api(StatusCode::BAD_REQUEST, e.to_string())),
        };

        let person: Person = body.into();

        let put_item_request = self
            .dynamodb_client
            .put_item()
            .table_name(self.table_name.clone())
            .item("id", AttributeValue::S(person.id.clone()))
            .item("firstName", AttributeValue::S(person.first_name.clone()))
            .item("lastName", AttributeValue::S(person.last_name.clone()))
            .item("age", AttributeValue::N(person.age.to_string()))
            .send()
            .await;

        match put_item_request {
            Ok(_) => {
                let resp = response::api(StatusCode::OK, json!({ "person": person }));
                Ok(resp)
            }
            Err(_) => {
                let resp = response::api(
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
