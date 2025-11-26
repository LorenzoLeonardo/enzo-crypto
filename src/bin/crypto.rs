use async_trait::async_trait;
use base64::{Engine, engine::general_purpose};
use ipc_broker::worker::{SharedObject, WorkerBuilder};
use serde_json::{Value, json};
use simple_enc_dec::{decrypt, encrypt};

#[derive(serde::Deserialize)]
struct Param {
    #[serde(default)]
    input: String,
    #[serde(default)]
    passphrase: String,
}

struct Crypto;

impl Crypto {
    /// Wrap Ok(T) or Err(E) into a JSON result
    fn wrap_result<T, E>(res: Result<T, E>) -> Value
    where
        T: serde::Serialize,
        E: std::fmt::Display,
    {
        res.map(|v| json!({ "result": v }))
            .unwrap_or_else(|e| json!({ "error": e.to_string() }))
    }

    /// Base64 decode helper
    fn decode_base64(input: &str) -> Value {
        Self::wrap_result(
            general_purpose::STANDARD
                .decode(input)
                .map(|bytes| bytes.into_iter().map(|b| b as char).collect::<String>()),
        )
    }

    /// Base64 encode helper
    fn encode_base64(input: &str) -> Value {
        json!({
            "result": general_purpose::STANDARD.encode(input)
        })
    }

    /// Require passphrase or return error JSON
    fn require_passphrase(passphrase: &str) -> Option<Value> {
        if passphrase.is_empty() {
            Some(json!({ "error": "Passphrase is required" }))
        } else {
            None
        }
    }
}

#[async_trait]
impl SharedObject for Crypto {
    async fn call(&self, method: &str, args: &Value) -> Value {
        let param: Param = match serde_json::from_value(args.clone()) {
            Ok(p) => p,
            Err(e) => return json!({ "error": format!("Invalid arguments: {}", e) }),
        };

        match method {
            "decode" => Crypto::decode_base64(&param.input),
            "encode" => Crypto::encode_base64(&param.input),
            "encrypt" => {
                if let Some(err) = Crypto::require_passphrase(&param.passphrase) {
                    return err;
                }
                Crypto::wrap_result(encrypt(&param.input, &param.passphrase))
            }
            "decrypt" => {
                if let Some(err) = Crypto::require_passphrase(&param.passphrase) {
                    return err;
                }
                Crypto::wrap_result(decrypt(&param.input, &param.passphrase))
            }

            _ => json!({ "error": "Unknown method" }),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Your async main function code here

    WorkerBuilder::new()
        .add("applications.crypto", Crypto)
        .spawn()
        .await?;

    Ok(())
}
