use std::{borrow::Cow, path::Path};

use async_trait::async_trait;
use base64::{Engine, engine::general_purpose};
use chrono::Local;
use enzo_crypto::{base52::Base52Codec, decrypt, encrypt, scrypt};
use fern::Dispatch;
use ipc_broker::worker::{SharedObject, WorkerBuilder};
use log::LevelFilter;
use serde_json::{Value, json};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[repr(i32)]
#[derive(Serialize_repr, Deserialize_repr, Debug, Default)]
enum Code {
    #[default]
    Success = 0,
    Error = -1,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CryptoResult<'a> {
    code: Code,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Cow<'a, str>>,
}

impl<'a> CryptoResult<'a> {
    fn success(result: String) -> Self {
        CryptoResult {
            code: Code::Success,
            result: Some(Cow::Owned(result.to_owned())),
            error: None,
        }
    }

    fn error(error: String) -> Self {
        CryptoResult {
            code: Code::Error,
            result: None,
            error: Some(Cow::Owned(error.to_owned())),
        }
    }
}

// Convert CryptoResult into serde_json::Value reliably
impl<'a> From<CryptoResult<'a>> for Value {
    fn from(cr: CryptoResult<'a>) -> Self {
        serde_json::to_value(cr).unwrap()
    }
}

#[derive(serde::Deserialize)]
struct Param<'a> {
    #[serde(default)]
    input: Cow<'a, str>,
    #[serde(default)]
    passphrase: Cow<'a, str>,
}

struct Crypto;

impl Crypto {
    /// Wrap Ok(T) or Err(E) into a JSON result
    fn wrap_result<T, E>(res: Result<T, E>) -> Value
    where
        T: std::fmt::Display,
        E: std::fmt::Display,
    {
        res.map(|v| CryptoResult::success(v.to_string()).into())
            .unwrap_or_else(|e| CryptoResult::error(e.to_string()).into())
    }

    /// Base64 decode helper
    fn decode_base64(input: &str) -> Value {
        log::info!("Decoding base64 input: {input}");
        Self::wrap_result(
            general_purpose::STANDARD
                .decode(input)
                .map(|bytes| bytes.into_iter().map(|b| b as char).collect::<String>()),
        )
    }

    /// Base64 encode helper
    fn encode_base64(input: &str) -> Value {
        log::info!("Encoding base64 input: {input}");
        CryptoResult::success(general_purpose::STANDARD.encode(input)).into()
    }

    /// Require passphrase or return error JSON
    fn require_passphrase(passphrase: &str) -> Option<Value> {
        if passphrase.is_empty() {
            Some(CryptoResult::error("Passphrase is required".to_string()).into())
        } else {
            None
        }
    }

    /// Base52 decode helper
    fn decode_base52(input: &str) -> Value {
        log::info!("Decoding base52 input: {input}");
        let codec = Base52Codec;

        Self::wrap_result(
            codec
                .decode(input)
                .map(|bytes| bytes.into_iter().map(|b| b as char).collect::<String>()),
        )
    }

    /// Base52 encode helper
    fn encode_base52(input: &str) -> Value {
        log::info!("Encoding base52 input: {input}");
        let codec = Base52Codec;
        CryptoResult::success(codec.encode(input)).into()
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
                log::info!("Encrypting input: {}", param.input);
                if let Some(err) = Crypto::require_passphrase(&param.passphrase) {
                    log::error!("Passphrase missing for encryption");
                    return err;
                }
                Crypto::wrap_result(encrypt(&param.input, &param.passphrase))
            }
            "decrypt" => {
                log::info!("Decrypting input: {}", param.input);
                if let Some(err) = Crypto::require_passphrase(&param.passphrase) {
                    log::error!("Passphrase missing for decryption");
                    return err;
                }
                Crypto::wrap_result(decrypt(&param.input, &param.passphrase))
            }
            "decode52" => Crypto::decode_base52(&param.input),
            "encode52" => Crypto::encode_base52(&param.input),
            "scrypt-encrypt" => {
                log::info!("Encrypting input: {}", param.input);
                if let Some(err) = Crypto::require_passphrase(&param.passphrase) {
                    log::error!("Passphrase missing for encryption");
                    return err;
                }
                Crypto::wrap_result(scrypt::encrypt_base64(
                    param.input.as_bytes(),
                    &param.passphrase,
                ))
            }
            "scrypt-decrypt" => {
                log::info!("Decrypting input: {}", param.input);
                if let Some(err) = Crypto::require_passphrase(&param.passphrase) {
                    log::error!("Passphrase missing for decryption");
                    return err;
                }
                Crypto::wrap_result(
                    scrypt::decrypt_base64(&param.input, &param.passphrase)
                        .map_err(|e| e.to_string())
                        .and_then(|bytes| String::from_utf8(bytes).map_err(|e| e.to_string())),
                )
            }
            _ => {
                let msg = format!("Unknown method called: {method}");
                log::warn!("{msg}");
                json!({ "error": msg })
            }
        }
    }
}

fn setup_logger() {
    let level_filter = match (Path::new("trace").exists(), Path::new("debug").exists()) {
        (true, true) | (true, false) => LevelFilter::Trace,
        (false, true) => LevelFilter::Debug,
        (false, false) => LevelFilter::Info, // Default level
    };

    if let Err(e) = Dispatch::new()
        .format(move |out, message, record| {
            let file = record.file().unwrap_or("unknown_file");
            let line = record.line().map_or(0, |l| l);

            match level_filter {
                LevelFilter::Off
                | LevelFilter::Error
                | LevelFilter::Warn
                | LevelFilter::Debug
                | LevelFilter::Trace => {
                    out.finish(format_args!(
                        "[{}][{}]: {} <{}:{}>",
                        Local::now().format("%b-%d-%Y %H:%M:%S.%f"),
                        record.level(),
                        message,
                        file,
                        line,
                    ));
                }
                LevelFilter::Info => {
                    out.finish(format_args!(
                        "[{}]: {} <{}:{}>",
                        record.level(),
                        message,
                        file,
                        line,
                    ));
                }
            }
        })
        .level(level_filter)
        .chain(std::io::stdout())
        .apply()
    {
        log::error!("Logger initialization failed: {e:?}");
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Your async main function code here
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");

    eprintln!("{name} has started v{version}...");

    setup_logger();
    WorkerBuilder::new()
        .add("applications.crypto", Crypto)
        .spawn()
        .await?;

    eprintln!("{name} has ended...");
    Ok(())
}
