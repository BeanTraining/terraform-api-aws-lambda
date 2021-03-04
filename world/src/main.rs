extern crate crypto;
extern crate rustc_serialize;

use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::sha2::Sha512;
use lambda_http::ext::PayloadError;
use lambda_http::{handler, lambda, Context, IntoResponse, Request, RequestExt};
use rustc_serialize::base64::{ToBase64, STANDARD};
use rustc_serialize::hex::ToHex;
use rustc_serialize::json::ToJson;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::fs::File;
use std::io::prelude::*;

type Error = Box<dyn std::error::Error + Sync + Send + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda::run(handler(world)).await?;
    Ok(())
}

async fn world(request: Request, _: Context) -> Result<impl IntoResponse, Error> {
    let secret = env::var("API_KEY").unwrap();

    let something_param = request.query_string_parameters();
    let headers = request.headers();
    let mut tfe_signature: String = String::from("");
    let mut header_names = "".to_owned();
    header_names.push_str(" <<<<<<<<<<<< START HEADERS >>>>>>>>>>>> ");
    for (header_name, header_value) in headers {
        header_names.push_str(header_name.as_str());
        header_names.push_str(" ---> ");
        header_names.push_str(header_value.to_str().unwrap_or(""));
        header_names.push_str(" <--- ");
        if header_name == "X-TFE-Notification-Signature" {
            tfe_signature = header_value.to_str().unwrap_or("").to_owned();
        }
    }
    header_names.push_str(" <<<<<<<<<<<< END HEADERS >>>>>>>>>>>> ");
    header_names.push_str(" <<<<<<< TOKEN IS HERE >>>>>> ");
    header_names.push_str(tfe_signature.as_str());

    let hmac_key = Vec::from("123");
    let message = request.body().to_json().to_string();
    println!("Message: {}", message);
    println!("HMAC key: {}", hmac_key.to_base64(STANDARD));
    let mut hmac = Hmac::new(Sha512::new(), &hmac_key);
    hmac.input(message.as_bytes());
    println!("HMAC digest: {}", hmac.result().code().to_hex());
    let hmac_hex = hmac.result().code().to_hex();

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

    let mut payload_content: String = String::from("");
    let payload: Payload;

    let _payload: Result<Option<Payload>, PayloadError> = request.payload();
    let _payload_unwrapped = _payload.unwrap_or_default();

    let body_content = if _payload_unwrapped.is_none() {
        payload = Payload {
            payload_version: 0,
            notification_configuration_id: "".to_string(),
            run_id: "".to_string(),
            run_message: "".to_string(),
            run_created_at: "".to_string(),
            run_created_by: "".to_string(),
            workspace_id: "".to_string(),
            workspace_name: "".to_string(),
            organization_name: "".to_string(),
        };
        " EMPTY BODY "
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
        payload_content.push_str(" HEXXXXXX ");
        payload_content.push_str(hmac_hex.as_str());
        payload_content.push_str(" >>><<< ");
        payload_content.push_str(" MSG ISSS ");
        payload_content.push_str(" >>><<< ");
        payload_content.push_str(message.as_str());

        payload_content.as_str()
    };
    let something = header_names
        + " ____________ "
        + secret.as_str()
        + " ____________ "
        + something_param.get("api_key").unwrap_or(" EMPTY API KEY ")
        + body_content
        + serde_json::to_string(&payload).unwrap().as_str();
    let mut file = File::create("/mnt/efs/foo.txt")?;
    file.write_all(something.as_bytes())?;

    // let contents = "hello world!!!!!!!";
    // `serde_json::Values` impl `IntoResponse` by default
    // creating an application/json response
    Ok(json!({
    "message": "Go Serverless v1.5! Your function executed successfully!",
    "contents": "From EFS ".to_owned() + &something
    }))
}
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[tokio::test]
//     async fn world_handles() {
//         let request = Request::default();
//         let expected = json!({
//         "message": "Go Serverless v1.3! Your function executed successfully!",
//         "contents": "From EFS "
//         })
//         .into_response();
//         let response = world(request, Context::default())
//             .await
//             .expect("expected Ok(_) value")
//             .into_response();
//         assert_eq!(response.body(), expected.body())
//     }
// }
//
