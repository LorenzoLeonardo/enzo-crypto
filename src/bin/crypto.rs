use std::path::Path;

use chrono::Local;
use enzo_crypto::crypto::Crypto;
use fern::Dispatch;
use ipc_broker::worker::WorkerBuilder;
use log::LevelFilter;
use tokio::{
    sync::mpsc::{self, unbounded_channel},
    time::Instant,
};

const TIMEOUT: u64 = 10;

struct LogHandler;

impl LogHandler {
    fn start() -> Self {
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
        let name = env!("CARGO_PKG_NAME");
        let version = env!("CARGO_PKG_VERSION");
        log::info!("{name} {version} has started...");
        Self
    }
}
impl Drop for LogHandler {
    fn drop(&mut self) {
        let name = env!("CARGO_PKG_NAME");
        let version = env!("CARGO_PKG_VERSION");
        log::info!("{name} {version} has ended...");
        log::logger().flush();
    }
}
// replace broken tail with a proper async main
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let logger = LogHandler::start();
    let (activity_tx, activity_rx) = unbounded_channel();

    let (builder, shutdown) = WorkerBuilder::new()
        .add("applications.crypto", Crypto::new(activity_tx))
        .with_graceful_shutdown();

    let handle = tokio::spawn(async move { builder.spawn().await });

    tokio::spawn(async move {
        run_with_inactivity_timeout(activity_rx).await;
        let _ = shutdown.send(true);
    });

    handle.await??;
    drop(logger);
    Ok(())
}

async fn run_with_inactivity_timeout(mut activity_rx: mpsc::UnboundedReceiver<()>) {
    let timeout = std::time::Duration::from_secs(TIMEOUT);
    let mut last_activity = Instant::now();

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                log::info!("Ctrl-C received, shutting down");
                break;
            }

            _ = tokio::time::sleep_until(last_activity + timeout) => {
                log::warn!("No activity for {TIMEOUT} seconds, shutting down");
                break;
            }

            msg = activity_rx.recv() => {
                match msg {
                    Some(()) => {
                        last_activity = Instant::now(); // ✅ reset timer
                    }
                    None => {
                        log::info!("Activity channel closed, shutting down");
                        break;
                    }
                }
            }
        }
    }
}
