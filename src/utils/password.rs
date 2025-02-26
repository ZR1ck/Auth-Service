use argon2::{
    password_hash::{self, rand_core::OsRng, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, PasswordHash,
};

pub fn hash_password(password: &str) -> Result<String, password_hash::errors::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let hash_pw = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok(hash_pw)
}

pub fn verify_password(password: &str, hash: &str) -> Result<(), password_hash::errors::Error> {
    let parsed_hash = PasswordHash::new(&hash)?;
    Argon2::default().verify_password(password.as_bytes(), &parsed_hash)
}
