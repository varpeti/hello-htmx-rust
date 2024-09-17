use std::{error::Error, thread::available_parallelism};

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Algorithm, Argon2, Params, Version,
};

/* https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html
    m=47104 (46 MiB), t=1
    m=19456 (19 MiB), t=2
    m=12288 (12 MiB), t=3
    m=9216 (9 MiB), t=4
    m=7168 (7 MiB), t=5
*/
const MEMORY_COST_KB: u32 = 46 * 1024;
const TIME_COST: u32 = 2;

fn get_a2id<'a>() -> Result<Argon2<'a>, Box<dyn Error>> {
    let num_of_available_paralelism: u32 = match available_parallelism() {
        Ok(num) => num.get() as u32,
        Err(_) => 1u32,
    };
    dbg!(&num_of_available_paralelism);
    let a2id: Argon2<'static> = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(MEMORY_COST_KB, TIME_COST, num_of_available_paralelism, None)?,
    );
    Ok(a2id)
}

pub fn hash_password(password: &str) -> Result<String, Box<dyn Error>> {
    let a2id = get_a2id()?;
    let salt = SaltString::generate(&mut OsRng);
    dbg!(&salt);
    let hash = a2id.hash_password(password.as_bytes(), &salt)?.to_string();
    dbg!(&hash);
    Ok(hash)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, Box<dyn Error>> {
    let a2id = get_a2id()?;
    let parsed_hash = PasswordHash::new(hash)?;
    dbg!(&parsed_hash);
    match a2id.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
