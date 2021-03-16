extern crate hyper;
extern crate hyper_tls;

use hyper::{Body as ClientRequestBody, Client, Request as ClientRequest};
use hyper_tls::HttpsConnector;
use lambda_http::ext::PayloadError;
use lambda_http::{handler, lambda, Context, IntoResponse, Request, RequestExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;

type Error = Box<dyn std::error::Error + Sync + Send + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda::run(handler(notification)).await?;
    Ok(())
}

#[derive(Deserialize, Serialize)]
struct Payload {
    payload_version: u8,
    notification_configuration_id: String,
    run_id: String,
    run_message: String,
    run_created_at: String,
    run_created_by: String,
    workspace_id: String,
    workspace_name: String,
    organization_name: String,
}

async fn notification(request: Request, _: Context) -> Result<impl IntoResponse, Error> {
    let secret = env::var("API_KEY").unwrap();
    let query_string_map = request.query_string_parameters();
    let incoming_api_key = query_string_map.get("api_key").unwrap_or(" EMPTY API KEY ");
    if secret != incoming_api_key {
        return Ok(json!({
        "message": "Unauthorised",
        "contents": "API KEY not matched!"
        }));
    }

    let payload: Payload;

    let _payload: Result<Option<Payload>, PayloadError> = request.payload();
    let _payload_unwrapped = _payload.unwrap_or_default();

    if _payload_unwrapped.is_none() {
        println!("EMPTY BODY");
    } else {
        payload = _payload_unwrapped.unwrap();
        println!("Body content: {}", serde_json::to_string(&payload).unwrap());
        let tfe_token = env::var("TFE_TOKEN").unwrap();
        let run_id = payload.run_id;
        apply_terraform_run(tfe_token, run_id).await?;
    };

    // `serde_json::Values` impl `IntoResponse` by default
    // creating an application/json response
    Ok(json!({
    "message": "Go Serverless v1.2! Your function executed successfully!",
    "contents": "From EFS " //.to_owned() + secret.as_str()
    }))
}

async fn apply_terraform_run(
    tfe_token: String,
    run_id: String,
) -> Result<impl IntoResponse, Error> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let req = ClientRequest::builder()
        .method("POST")
        // .uri("http://httpbin.org/post")
        .uri("https://app.terraform.io/api/v2/runs/".to_owned() + &run_id + "/actions/apply")
        .header("Authorization", "Bearer ".to_owned() + &tfe_token)
        .header("Content-Type", "application/vnd.api+json")
        .body(ClientRequestBody::from(
            "{\"comment\":\"Automatically approved from Lambda.\"}",
        ))
        .expect("request builder");
    let res = client.request(req).await?;
    // And then, if the request gets a response...
    println!("status: {}", res.status());

    // Concatenate the body stream into a single buffer...
    let buf = hyper::body::to_bytes(res).await?;

    println!("body: {:?}", buf);
    Ok(json!({
    "message": "TFE Run executed successfully!",
    "contents": "From Notification Handler " //.to_owned() + secret.as_str()
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn notification_handles() {
        env::set_var("API_KEY", "API_KEY_VALUE");
        env::set_var("TFE_TOKEN", "TFE_TOKEN_VALUE");
        let request = Request::default();
        let expected = json!({
        "message": "Unauthorised",
        "contents": "API KEY not matched!"
        })
        .into_response();
        let response = notification(request, Context::default())
            .await
            .expect("expected Ok(_) value")
            .into_response();
        assert_eq!(response.body(), expected.body())
    }
}
