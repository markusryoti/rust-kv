use rust_kv::kv::KV;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let kv = KV::new();

    match kv.listen() {
        Ok(_) => (),
        Err(e) => eprintln!("{}", e),
    }

    Ok(())
}
