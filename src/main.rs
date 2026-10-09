use rust_kv::kv::KV;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let kv = KV::new(4);

    kv.listen().expect("to work");

    Ok(())
}
