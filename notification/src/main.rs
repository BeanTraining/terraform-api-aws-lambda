extern crate hex;
extern crate hmac;
extern crate hyper;
extern crate hyper_tls;
extern crate sha2;

use hmac::{Hmac, Mac, NewMac};
use hyper::{Body as ClientRequestBody, Client, Request as ClientRequest};
use hyper_tls::HttpsConnector;
use lambda_http::ext::PayloadError;
use lambda_http::http::{HeaderMap, HeaderValue};
use lambda_http::{handler, lambda, Context, IntoResponse, Request, RequestExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Sha512;
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
    if request.body().is_empty() {
        return Ok(json!({
        "message": "Go Serverless v1.2! Your function executed successfully!",
        "contents": "From EFS " //.to_owned() + secret.as_str()
        }));
    }
    let headers = request.headers();
    let api_key = env::var("API_KEY").unwrap();
    let message = serde_json::to_string(request.body()).unwrap();

    if !validate_signature(&api_key, &message, get_signature_from_headers(headers)) {
        return Err(Error::from("Unauthorised"));
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

fn get_signature_from_headers(headers: &HeaderMap<HeaderValue>) -> &str {
    let mut tfe_signature = "";

    for (header_name, header_value) in headers {
        println!(" <<<<<<<<<<< ");
        println!("HEADER NAME: {}", header_name.as_str());
        println!("HEADER VALUE: {}", header_value.to_str().unwrap_or(""));
        println!(" >>>>>>>>>>> ");
        if header_name == "X-TFE-Notification-Signature" {
            tfe_signature = header_value.to_str().unwrap_or("");
        }
    }
    tfe_signature
}

fn validate_signature(secret: &str, message: &str, tfe_signature: &str) -> bool {
    let secret_in_bytes = secret.as_bytes();
    println!("Message: {}", message);
    println!("HMAC key: {}", secret);
    println!("Signature is {}", tfe_signature);
    // Create alias for HMAC-SHA256
    type HmacSha512 = Hmac<Sha512>;

    // Create HMAC-SHA256 instance which implements `Mac` trait
    let mut mac = HmacSha512::new_varkey(secret_in_bytes).expect("HMAC can take key of any size");
    mac.update(message.as_bytes());

    // `result` has type `Output` which is a thin wrapper around array of
    // bytes for providing constant time equality check
    let hmac_result = mac.finalize();
    // To get underlying array use `into_bytes` method, but be careful, since
    // incorrect use of the code value may permit timing attacks which defeat
    // the security provided by the `Output`
    //     let code_bytes = result.into_bytes();
    let hmac_result_in_bytes = hmac_result.into_bytes();
    let hmac_result_in_hex = hex::encode(hmac_result_in_bytes);
    println!("hmac_result_in_hex is {:?}", hmac_result_in_hex);
    hmac_result_in_hex == tfe_signature
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
    use lambda_http::Body;

    #[tokio::test]
    async fn when_invalid_api_key_return_unauthorised_error() {
        env::set_var("API_KEY", "API_KEY_VALUE");
        env::set_var("TFE_TOKEN", "TFE_TOKEN_VALUE");
        let request = Request::new(Body::from("Test Invalid Json Body"));
        let response = notification(request, Context::default()).await;
        assert_eq!(response.err().unwrap().to_string(), "Unauthorised")
    }
}
