use argon2::{
    password_hash::{self, rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};

pub fn hash_password(password: String) -> Result<String, password_hash::errors::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let hash_pw = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok(hash_pw)
}
