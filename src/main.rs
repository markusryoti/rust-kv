use std::sync::Arc;

use rust_kv::kv::KV;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let kv = Arc::new(KV::new());

    kv.listen(8).expect("to work");

    Ok(())
}
