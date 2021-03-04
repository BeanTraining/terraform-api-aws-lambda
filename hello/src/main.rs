use lambda::{handler_fn, Context};
use serde_json::Value;
use std::fs::File;
use std::io::prelude::*;

type Error = Box<dyn std::error::Error + Sync + Send + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda::run(handler_fn(hello)).await?;
    Ok(())
}

async fn hello(event: Value, _: Context) -> Result<Value, Error> {
    let is_mock_event = event["isMock"].as_bool().unwrap_or(false);
    if is_mock_event {
        return Ok(event);
    }
    let mut file = File::open("/mnt/efs/foo.txt")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(Value::from(contents))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn hello_handles() {
        let event = json!({
            "answer": 42,
            "isMock": true
        });
        assert_eq!(
            hello(event.clone(), Context::default())
                .await
                .expect("expected Ok(_) value"),
            event
        )
    }
}
