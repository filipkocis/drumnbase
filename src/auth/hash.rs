use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};

pub struct Hashish;

impl Hashish {
    /// Hash a password, returns a PHC string
    pub fn hash(password: &str) -> Result<String, String> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt).map_err(|e| e.to_string())?
            .to_string();

        Ok(password_hash)
    }

    /// Verify a password against a PHC string
    pub fn verify(password: &str, hash: &str) -> Result<bool, String> {
        let password_hash = PasswordHash::new(hash).map_err(|e| e.to_string())?;
        let verified = Argon2::default()
            .verify_password(password.as_bytes(), &password_hash)
            .is_ok();

        Ok(verified)
    }
}
