use aes::cipher::{BlockCipherDecrypt, KeyInit, KeyIvInit, StreamCipher};
use anyhow::{Result, anyhow};
use std::fs;

type Aes256Ctr = ctr::Ctr128BE<aes::Aes256>;

/// AES Key Wrap unwrap, per RFC 3394 §2.2.2, with a caller-supplied 64-bit IV
/// instead of the default A6A6A6A6A6A6A6A6.
fn aes_kw_unwrap(kek: &[u8], wrapped: &[u8], expected_iv: &[u8; 8]) -> Result<Vec<u8>> {
    if wrapped.len() < 16 || wrapped.len() % 8 != 0 {
        return Err(anyhow!(
            "wrapped key must be a multiple of 8 bytes and at least 16"
        ));
    }
    let kek: [u8; 32] = kek.try_into().map_err(|_| anyhow!("kek must be 32 bytes"))?;
    let cipher = aes::Aes256::new(&kek.into());

    let n = wrapped.len() / 8 - 1;
    let mut a: [u8; 8] = wrapped[..8].try_into().unwrap();
    let mut r: Vec<[u8; 8]> = (0..n)
        .map(|i| wrapped[8 * (i + 1)..8 * (i + 2)].try_into().unwrap())
        .collect();

    for j in (0..=5u64).rev() {
        for i in (1..=n).rev() {
            let t = (n as u64) * j + (i as u64);
            let t_bytes = t.to_be_bytes();

            let mut block = [0u8; 16];
            for k in 0..8 {
                block[k] = a[k] ^ t_bytes[k];
            }
            block[8..].copy_from_slice(&r[i - 1]);

            cipher.decrypt_block((&mut block).into());

            a.copy_from_slice(&block[..8]);
            r[i - 1].copy_from_slice(&block[8..]);
        }
    }

    if &a != expected_iv {
        return Err(anyhow!(
            "integrity check failed: recovered A = {:02x?}, expected {:02x?}",
            a,
            expected_iv
        ));
    }

    Ok(r.into_iter().flatten().collect())
}

fn decrypt_256_ctr(key: &[u8], iv: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>> {
    let key: [u8; 32] = key.try_into().map_err(|_| anyhow!("key must be 32 bytes"))?;
    let iv: [u8; 16] = iv.try_into().map_err(|_| anyhow!("iv must be 16 bytes"))?;

    let mut buf = ciphertext.to_vec();
    let mut cipher = Aes256Ctr::new(&key.into(), &iv.into());
    cipher.apply_keystream(&mut buf);
    Ok(buf)
}

fn decode_layer5(payload: Vec<u8>) -> Result<Vec<u8>> {
    let mut index = 0;

    let kek = &payload[0..32];
    index += 32;
    let iv_key: &[u8; 8] = payload[index..(index + 8)].try_into().unwrap();
    index += 8;
    let encrypted_key = &payload[index..(index + 40)];

    let key = aes_kw_unwrap(kek, encrypted_key, iv_key)?;

    index += 40;
    let iv_payload = &payload[index..(index + 16)];
    index += 16;

    let encrypted_payload = &payload[index..];
    decrypt_256_ctr(&key, iv_payload, encrypted_payload)
}

fn main() -> Result<()> {
    let input = fs::read_to_string("payload/layer5.data")?;
    let ascii85_decoded = ascii85::decode(&input).map_err(|e| anyhow!("{e:?}"))?;

    let decoded = decode_layer5(ascii85_decoded)?;

    fs::write("layer5_out.data", &decoded)?;

    Ok(())
}
