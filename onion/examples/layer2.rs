use std::error::Error;
use std::fs;

struct Decoder {
    data: Vec<u8>,
    acc: u64,
    bytes_acc: u8,
}

impl Decoder {
    fn new() -> Self {
        Self {
            data: Vec::new(),
            acc: 0,
            bytes_acc: 0,
        }
    }

    fn push(&mut self, byte: u8) {
        // Push when parity is 1
        let parity_bit = byte & 1;
        let data_bits = byte >> 1;
        let parity = data_bits.count_ones() as u8 & 1;
        if parity == parity_bit {
            // Accumulate in a u64
            self.acc = (self.acc << 7) | data_bits as u64;
            self.bytes_acc += 1;
        }
        if self.bytes_acc == 8 {
            self.decode_acc();
        }
    }

    fn decode_acc(&mut self) {
        // Read from u64 into
        let mut out = [0u8; 7];
        for i in 0..7 {
            out[i] = (self.acc >> (8 * (6 - i)) & 0xFF) as u8;
        }
        self.data.extend_from_slice(&out);
        self.bytes_acc = 0;
        self.acc = 0;
    }
}

fn decode_layer2(data: Vec<u8>) -> Vec<u8> {
    let mut decoder = Decoder::new();

    for byte in data.iter() {
        decoder.push(*byte);
    }

    decoder.data
}

fn main() -> Result<(), Box<dyn Error>> {
    let input = fs::read_to_string("payload/layer2.data")?;
    let ascii85_decoded = ascii85::decode(&input)?;

    // This could also be done by mutating the decoded ascii85 array
    let decoded = decode_layer2(ascii85_decoded);

    fs::write("layer2_out.data", &decoded)?;

    Ok(())
}
