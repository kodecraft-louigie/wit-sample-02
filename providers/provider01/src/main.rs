//! NATS implementation for wasmcloud:messaging.

mod connection;
mod nats;

use nats::NatsMessagingProvider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    eprintln!("[From eprintln!]: provider01 starting...");
    NatsMessagingProvider::run().await?;
    eprintln!("provider01 exiting");
    Ok(())
}
