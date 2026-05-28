use std::error::Error;
use std::fs;

fn decode_layer1(data: Vec<u8>) -> Vec<u8> {
    let mut decoded = Vec::with_capacity(data.len());
    for byte in data.iter() {
        // Mask to flip
        let mut decoded_byte = *byte ^ (0b01010101 as u8);

        // Rotate right
        decoded_byte = (decoded_byte & 1) + (decoded_byte >> 1);
        decoded.push(decoded_byte);
    }
    decoded
}

fn main() -> Result<(), Box<dyn Error>> {
    let input = fs::read_to_string("payload/layer1.data")?;
    let ascii85_decoded = ascii85::decode(&input)?;

    // This could also be done by mutating the decoded ascii85 array
    let decoded = decode_layer1(ascii85_decoded);

    fs::write("layer1_out.data", &decoded)?;

    Ok(())
}
