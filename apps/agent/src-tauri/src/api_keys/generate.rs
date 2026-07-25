pub fn generate_key_id() -> String {
    format!("key_{}", random_token(8))
}

pub fn generate_secret_key() -> String {
    format!("sk-selfapi-{}", random_token(24))
}

fn random_token(byte_len: usize) -> String {
    let mut buf = vec![0u8; byte_len];
    getrandom::getrandom(&mut buf).expect("OS random source unavailable");
    buf.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_key_has_expected_prefix() {
        let key = generate_secret_key();
        assert!(key.starts_with("sk-selfapi-"));
        assert!(key.len() > 20);
    }
}

