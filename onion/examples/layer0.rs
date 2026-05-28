use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    let input = fs::read_to_string("payload/onion.data")?;
    let decoded = ascii85::decode(&input)?;
    fs::write("layer0_out.data", &decoded)?;
    println!("Wrote {} bytes to layer0_out.data", decoded.len());
    Ok(())
}
