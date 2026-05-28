use std::{error::Error, fmt, fs};

#[derive(Debug)]
struct Ipv4Header {
    _ihl: u8, // header length in bytes
    total_length: u16,
    protocol: u8,
    _ttl: u8,
    src: [u8; 4],
    dst: [u8; 4],
}

#[derive(Debug)]
struct UdpHeader {
    src_port: u16,
    dst_port: u16,
    length: u16,
    checksum: u16,
}

fn checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;

    // sum 16-bit big-endian words
    let mut chunks = data.chunks_exact(2);
    for c in &mut chunks {
        sum += u16::from_be_bytes([c[0], c[1]]) as u32;
    }
    // odd trailing byte, if any (padded on the right with zero)
    if let Some(&[b]) = Some(chunks.remainder()) {
        sum += (b as u32) << 8;
    }

    // fold carries into the low 16 bits
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }

    !(sum as u16)
}

#[derive(Debug)]
enum ParseError {
    Unknown(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Unknown(s) => write!(f, "unknown error {s}"),
        }
    }
}

impl Error for ParseError {}

impl Ipv4Header {
    fn parse_ipv4(data: &[u8]) -> Result<Ipv4Header, ParseError> {
        let version = data[0] >> 4;
        if version != 4 {
            return Err(ParseError::Unknown("corrupt header".into()));
        }

        let ihl = (data[0] & 0x0F) as usize * 4;
        if ihl < 20 || data.len() < ihl {
            return Err(ParseError::Unknown("malformed or truncated".into()));
        }

        let header = Ipv4Header {
            _ihl: ihl as u8,
            total_length: u16::from_be_bytes([data[2], data[3]]),
            _ttl: data[8],
            protocol: data[9],
            src: [data[12], data[13], data[14], data[15]],
            dst: [data[16], data[17], data[18], data[19]],
        };

        assert!(header.protocol == 17);

        Ok(header)
    }
}

impl UdpHeader {
    fn parse_udp(data: &[u8]) -> (UdpHeader, &[u8]) {
        let header = UdpHeader {
            src_port: u16::from_be_bytes([data[0], data[1]]),
            dst_port: u16::from_be_bytes([data[2], data[3]]),
            length: u16::from_be_bytes([data[4], data[5]]),
            checksum: u16::from_be_bytes([data[6], data[7]]),
        };

        (header, &data[8..])
    }

    fn checksum_valid(&self, ip: &Ipv4Header, payload: &[u8]) -> bool {
        if self.checksum == 0 {
            return true; // checksum not used by sender
        }

        // reconstruct the UDP segment: 8-byte header + payload
        let mut segment = Vec::with_capacity(8 + payload.len());
        segment.extend_from_slice(&self.src_port.to_be_bytes());
        segment.extend_from_slice(&self.dst_port.to_be_bytes());
        segment.extend_from_slice(&self.length.to_be_bytes());
        segment.extend_from_slice(&self.checksum.to_be_bytes());
        segment.extend_from_slice(payload);

        // pseudo-header + segment
        let mut buf = Vec::with_capacity(12 + segment.len());
        buf.extend_from_slice(&ip.src);
        buf.extend_from_slice(&ip.dst);
        buf.push(0);
        buf.push(17);
        buf.extend_from_slice(&(segment.len() as u16).to_be_bytes());
        buf.extend_from_slice(&segment);

        checksum(&buf) == 0
    }
}

fn decode_layer4(packets: Vec<u8>) -> Result<Vec<u8>, ParseError> {
    let mut index: usize = 0;
    let mut payload = Vec::new();
    let mut qualifing_count = 0;
    let mut packet_count = 0;

    while index < packets.len() {
        let ipv4_header_raw = &packets[index..index + 20];
        let ipv4_header = Ipv4Header::parse_ipv4(ipv4_header_raw)?;
        index += 20;

        let udf_packet_len = (ipv4_header.total_length - 20) as usize;

        let (udp_header, udp_data) =
            UdpHeader::parse_udp(&packets[index..(index + udf_packet_len)]);

        index += udf_packet_len;
        packet_count += 1;

        if checksum(&ipv4_header_raw) != 0 {
            continue;
        }

        if !udp_header.checksum_valid(&ipv4_header, udp_data) {
            continue;
        }

        // println!("Packet {} Qualifing {}", packet_count, qualifing_count);
        // println!("IPV4 Header {:?}", ipv4_header);
        // println!("UDP Header {:?}", udp_header);

        if (ipv4_header.src[0] == 10
            && ipv4_header.src[1] == 1
            && ipv4_header.src[2] == 1
            && ipv4_header.src[3] == 10)
            && (ipv4_header.dst[0] == 10
                && ipv4_header.dst[1] == 1
                && ipv4_header.dst[2] == 1
                && ipv4_header.dst[3] == 200)
            && udp_header.dst_port == 42069
        {
            payload.extend_from_slice(&udp_data);
            qualifing_count += 1;
        }
    }

    println!(
        "Packets {} Total qualifiying packets {}",
        packet_count, qualifing_count
    );

    Ok(payload.to_vec())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input = fs::read_to_string("payload/layer4.data")?;
    let ascii85_decoded = ascii85::decode(&input)?;

    // This could also be done by mutating the decoded ascii85 array
    let decoded = decode_layer4(ascii85_decoded)?;

    fs::write("layer4_out.data", &decoded)?;

    Ok(())
}
