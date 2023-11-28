use anyhow::Result;
use argon2::{
    password_hash::{rand_core::OsRng, Error, PasswordHasher, SaltString},
    Argon2, PasswordVerifier,
};
use csrf::{ChaCha20Poly1305CsrfProtection, CsrfProtection};
use data_encoding::BASE64;

lazy_static! {
    pub static ref AEAD_KEY: [u8; 32] = {
        let aead_key_var = dotenv::var("AEAD_KEY").expect("AEAD_KEY not found in .env file!");
        match aead_key_var.as_bytes().try_into() {
            Ok(key) => {
                return key;
            }
            Err(e) => {
                eprintln!("Failure converting AEAD_KEY into array: {}", e);
                std::process::exit(1);
            }
        }
    };
}

const ONE_WEEK_SECONDS: u32 = 10;

fn sprinkle_pepper(pw_input: &str) -> String {
    let pepper = dotenv::var("PW_PEPPER").expect("PW_PWPPER not found in ENV!");

    return format!("{pepper}{pw_input}");
}

fn get_protection() -> impl CsrfProtection {
    return ChaCha20Poly1305CsrfProtection::from_key(*AEAD_KEY);
}

pub fn get_new_token_pair() -> Result<(String, String)> {
    let protect = get_protection();
    let (token, cookie) = protect.generate_token_pair(None, ONE_WEEK_SECONDS.into())?;

    return Ok((token.b64_string(), cookie.b64_string()));
}

pub fn verify_cookie(cookie: &str) -> Result<()> {
    let protect = get_protection();

    let cookie_bytes = BASE64.decode(cookie.as_bytes())?;
    let raw = protect.parse_cookie(&cookie_bytes)?;

    return Ok(());
}

pub fn verify_token_pair(token: &str, cookie: &str) -> Result<()> {
    let protect = get_protection();
    return Ok(());
}

pub fn get_new_hashed_password(pw_input: &str) -> Result<String, Error> {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);

    let hashed = argon2
        .hash_password(sprinkle_pepper(pw_input).as_bytes(), &salt)?
        .to_string();

    return Ok(hashed);
}

pub fn verify_password(candidate_hash: &str, candidate_pw: &str) -> Result<(), Error> {
    let argon2 = Argon2::default();
    let pw_hash = argon2::PasswordHash::new(&candidate_hash)?;
    return argon2.verify_password(sprinkle_pepper(candidate_pw).as_bytes(), &pw_hash);
}
