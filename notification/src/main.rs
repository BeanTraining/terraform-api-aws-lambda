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
        "contents": "API KEY not matched "
        }));
    }

    let mut payload_content: String = String::from("");
    let payload: Payload;

    let _payload: Result<Option<Payload>, PayloadError> = request.payload();
    let _payload_unwrapped = _payload.unwrap_or_default();

    if _payload_unwrapped.is_none() {
        println!("EMPTY BODY");
    } else {
        payload = _payload_unwrapped.unwrap();
        payload_content.push_str(" >>><<< ");
        payload_content.push_str(payload.payload_version.to_string().as_str());
        payload_content.push_str(" >>><<< ");
        payload_content.push_str(payload.notification_configuration_id.as_str());
        payload_content.push_str(" >>><<< ");
        payload_content.push_str(payload.run_id.as_str());
        payload_content.push_str(" >>><<< ");
        payload_content.push_str(payload.run_message.as_str());
        payload_content.push_str(" >>><<< ");
        payload_content.push_str(payload.run_created_at.as_str());
        payload_content.push_str(" >>><<< ");
        payload_content.push_str(payload.run_created_by.as_str());
        payload_content.push_str(" >>><<< ");
        payload_content.push_str(payload.workspace_id.as_str());
        payload_content.push_str(" >>><<< ");
        payload_content.push_str(payload.workspace_name.as_str());
        payload_content.push_str(" >>><<< ");
        payload_content.push_str(payload.organization_name.as_str());
        payload_content.push_str(" >>><<< ");
        println!("Body content: {}", payload_content.as_str());
    };

    // let contents = "hello world!!!!!!!";
    // `serde_json::Values` impl `IntoResponse` by default
    // creating an application/json response
    Ok(json!({
    "message": "Go Serverless v1.3! Your function executed successfully!",
    "contents": "From EFS " //.to_owned() + secret.as_str()
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn notification_handles() {
        env::set_var("API_KEY", "API_KEY");
        let request = Request::default();
        let expected = json!({
        "message": "Unauthorised",
        "contents": "API KEY not matched "
        })
        .into_response();
        let response = notification(request, Context::default())
            .await
            .expect("expected Ok(_) value")
            .into_response();
        assert_eq!(response.body(), expected.body())
    }
}
