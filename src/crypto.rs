
use {
    crate::error::Error,
    argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2,
    },
    chacha20poly1305::{
        aead::{Aead, KeyInit, OsRng as CryptoRng},
        ChaCha20Poly1305,
        Nonce,
    },
    rand::RngCore,
    serde::{Deserialize, Serialize},
    zeroize::Zeroize,
};


#[derive(Serialize, Deserialize)]
pub struct EncryptedData {
    pub cipher_text: Vec<u8>,
    pub salt: String,
    pub nonce: Vec<u8>,
}

pub fn encrypt_password(password: &str, master_key: &[u8]) -> Result<EncryptedData, Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let key = argon2
        .hash_password(master_key, &salt)?
        .to_string();

    let cipher = ChaCha20Poly1305::new_from_slice(key.as_bytes())?;
    let mut nonce = [0u8; 12];
    CryptoRng.fill_bytes(&mut nonce);

    let nonce = Nonce::from_slice(&nonce);
    let cipher_text = cipher.encrypt(nonce, password.as_bytes())?;

    Ok(EncryptedData {
        cipher_text,
        salt: salt.to_string(),
        nonce: nonce.to_vec(),
    })

}

pub fn decrypt_password(data: &EncryptedData, master_key: &[u8]) -> Result<String, Error> {
    let argon2 = Argon2::default();
    let key = argon2
        .hash_password(master_key, &data.salt)?
        .to_string();

    let cipher = ChaCha20Poly1305::new_from_slice(key.as_bytes())?;
    let nonce = Nonce::from_slice(&data.nonce);

    let plain_text = cipher.decrypt(nonce, data.cipher_text.as_ref())?;

    String::from_utf8(plain_text).map_err(|e| Error::Crypto(e.to_string()))
}

