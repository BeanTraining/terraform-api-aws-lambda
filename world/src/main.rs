extern crate hex;
extern crate hmac;
extern crate rustc_serialize;
extern crate sha2;

use hmac::{Hmac as MyHmac, Mac as MyMac, NewMac};
use lambda_http::ext::PayloadError;
use lambda_http::{handler, lambda, Context, IntoResponse, Request, RequestExt};
use rustc_serialize::base64::{ToBase64, STANDARD};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Sha512 as MySha512;
// use std::env;
// use std::fs::File;
// use std::io::prelude::*;

type Error = Box<dyn std::error::Error + Sync + Send + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda::run(handler(world)).await?;
    Ok(())
}

async fn world(request: Request, _: Context) -> Result<impl IntoResponse, Error> {
    // let secret = env::var("API_KEY").unwrap();
    // let something_param = request.query_string_parameters();
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

    let hmac_key = b"123";
    // let message = "{\"payload_version\":1,\"notification_configuration_id\":\"nc-HUF6ozX14EHzjB8p\",\"run_url\":\"https://app.terraform.io/app/BeanTraining/sg-dev-main-apps-example/runs/run-sabKBrwfmk8mWBHt\",\"run_id\":\"run-sabKBrwfmk8mWBHt\",\"run_message\":\"Queued manually in Terraform Cloud\",\"run_created_at\":\"2021-03-16T15:19:01.000Z\",\"run_created_by\":\"peterbean410\",\"workspace_id\":\"ws-BpcTcWRAHe5L6akf\",\"workspace_name\":\"sg-dev-main-apps-example\",\"organization_name\":\"BeanTraining\",\"notifications\":[{\"message\":\"Run Planning\",\"trigger\":\"run:planning\",\"run_status\":\"planning\",\"run_updated_at\":\"2021-03-16T15:19:03.000Z\",\"run_updated_by\":null}]}".to_owned();
    let message = serde_json::to_string(request.body()).unwrap();
    println!("Message: {}", message);
    println!(
        "Body content: {}",
        serde_json::to_string(request.body()).unwrap()
    );
    println!("HMAC key: {:?}", hmac_key);

    let test_sig = "90ae95c5a871e584f8992ef15dd7fd0adba4086594827d7555d53a37cdc3354420e8a3c7627d233eade8229f02b13291d0ca0ec80d592a9670543d404b01a960";
    println!("Test_sig is: {:?}", test_sig);
    // Create alias for HMAC-SHA256
    type HmacSha256 = MyHmac<MySha512>;

    // Create HMAC-SHA256 instance which implements `Mac` trait
    let mut mac = HmacSha256::new_varkey(hmac_key).expect("HMAC can take key of any size");
    mac.update(message.as_bytes());

    // `result` has type `Output` which is a thin wrapper around array of
    // bytes for providing constant time equality check
    let my_result = mac.finalize();

    // To get underlying array use `into_bytes` method, but be careful, since
    // incorrect use of the code value may permit timing attacks which defeat
    // the security provided by the `Output`
    //     let code_bytes = result.into_bytes();
    println!("Signature is {}", tfe_signature);
    let my_result_in_bytes = my_result.into_bytes();
    let r2 = hex::encode(my_result_in_bytes);
    println!("r2 is {:?}", r2);
    println!(
        "my_result in base64 is {:?}",
        my_result_in_bytes.to_base64(STANDARD)
    );
    println!("my_result is {:?}", my_result_in_bytes);
    println!("Signature in bytes {:?}", tfe_signature.as_bytes());
    // println!("mac hex is {}", mac.finalize().code().to_hex());

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
        message
    } else {
        payload = _payload_unwrapped.unwrap();
        payload_content.push_str(serde_json::to_string(&payload).unwrap().as_str());

        payload_content
    };

    // let mut file = File::create("/mnt/efs/foo.txt")?;
    // file.write_all(something.as_bytes())?;

    // let contents = "hello world!!!!!!!";
    // `serde_json::Values` impl `IntoResponse` by default
    // creating an application/json response
    Ok(json!({
    "message": "Go Serverless v1.6! In world function. Your function executed successfully!",
    "contents": body_content
    }))
}
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[tokio::test]
//     async fn world_handles() {
//         env::set_var("API_KEY", "123");
//         env::set_var("TFE_TOKEN", "TFE_TOKEN_VALUE");
//
//         let request = Request::default();
//         let expected = json!({
//             "message": "Go Serverless v1.6! In world function. Your function executed successfully!",
//             "contents": "From EFS"
//         })
//         .into_response();
//         let response = world(request, Context::default())
//             .await
//             .expect("expected Ok(_) value")
//             .into_response();
//         assert_eq!(response.body(), expected.body())
//     }
// }
