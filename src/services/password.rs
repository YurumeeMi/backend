use argon2::Argon2;
use argon2::password_hash::phc::PasswordHash;
use argon2::password_hash::{PasswordHasher, PasswordVerifier};

pub fn hash_password(password: &str) -> Result<String, String> {
    Argon2::default()
        .hash_password(password.as_bytes())
        .map(|hash| hash.to_string())
        .map_err(|err| err.to_string())
}

pub fn password_verify(password: &str, stored_hash: &str) -> Result<(), String> {
    let parsed_hash = PasswordHash::new(stored_hash).map_err(|err| err.to_string())?;

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|err| err.to_string())
}

#[test]
fn benchmark_password_hash() {
    use std::time::Instant;

    let start = Instant::now();
    let _hash = hash_password("benchmark-password").unwrap();

    println!("Hash Time: {:?}", start.elapsed());
}
