use std::{error::Error, thread::available_parallelism};

use argon2::{
    password_hash::{
        rand_core::{OsRng, RngCore},
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Algorithm, Argon2, Params, Version,
};
use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, Message,
    SmtpTransport, Transport,
};
use serde::{Deserialize, Serialize};

use crate::{
    clients::{add_connection, Clients},
    config::EmailConfig,
    handle_websocket::{CID, TX},
    uuser::{Auth, Uuser},
    DB,
};

/* https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html
    m=47104 (46 MiB), t=1
    m=19456 (19 MiB), t=2
    m=12288 (12 MiB), t=3
    m=9216 (9 MiB), t=4
    m=7168 (7 MiB), t=5
*/
const MEMORY_COST_MB: u32 = 46;
const TIME_COST: u32 = 2;
const ALPHABET: [char; 32] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U',
    'V', 'W', 'X', 'Y', 'Z', '2', '3', '4', '5', '6', '7', '8', '9',
];
const CODE_LEN: usize = 8;

fn get_a2id<'a>() -> Result<Argon2<'a>, Box<dyn Error>> {
    let num_of_available_paralelism: u32 = match available_parallelism() {
        Ok(num) => num.get() as u32,
        Err(_) => 1u32,
    };
    let a2id: Argon2<'static> = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(
            MEMORY_COST_MB * 1024,
            TIME_COST,
            num_of_available_paralelism,
            None,
        )?,
    );
    Ok(a2id)
}

pub fn hash_password(password: &str) -> Result<String, Box<dyn Error>> {
    let a2id = get_a2id()?;
    let salt = SaltString::generate(&mut OsRng);
    let hash = a2id.hash_password(password.as_bytes(), &salt)?.to_string();
    Ok(hash)
}

fn verify_password(password: &str, hash: &str) -> bool {
    // Box<dyn Error> cannot be used due Send limitations with async...
    let a2id = match get_a2id() {
        Ok(a2id) => a2id,
        Err(err) => {
            eprintln!("Error: argon2id error! {}", err);
            return false;
        }
    };
    let parsed_hash = match PasswordHash::new(hash) {
        Ok(parsed_hash) => parsed_hash,
        Err(err) => {
            eprintln!("Error: Hash parsing error! {}", err);
            return false;
        }
    };
    a2id.verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
}

pub async fn login_with_password(
    login_form: LoginForm,
    tx: TX,
    db: DB,
    clients: Clients,
    client_id: CID,
) -> Result<(), Box<dyn Error>> {
    let uuser = match Uuser::select_where(&db, "email = $1", &[&login_form.email])
        .await?
        .values()
        .next()
    {
        Some(uuser) => uuser.clone(),
        None => Err(format!(
            "Error: Uuser not found by ({}) email!",
            login_form.email
        ))?,
    };

    let auth = match Auth::select_where(&db, "uuser_key = $1", &[&uuser.id])
        .await?
        .values()
        .next()
    {
        Some(auth) => auth.clone(),
        None => Err(format!(
            "Error auth not found for uuser (by email: {})!",
            login_form.email
        ))?,
    };

    match verify_password(&login_form.password, &auth.phc_string) {
        true => add_connection(clients.clone(), uuser.id, tx).await?,
        false => Err("Invalid password!")?,
    };

    let mut client_id = client_id.lock().await;
    *client_id = Some(uuser.id);

    Ok(())
}

pub async fn send_code_to_email(
    config: EmailConfig,
    to_email: String,
) -> Result<(), Box<dyn Error>> {
    let code = generate_human_readable_code()?;
    let email = Message::builder()
        .from(config.sender_email.parse()?)
        .to(to_email.parse()?)
        .subject("Verification Code")
        .header(ContentType::TEXT_PLAIN)
        .body(code)?;
    let creds = Credentials::new(config.username.clone(), config.password.clone());
    let mailer = SmtpTransport::relay(&config.smtp)?
        .credentials(creds)
        .build();

    mailer.send(&email)?;
    Ok(())
}

fn generate_human_readable_code() -> Result<String, Box<dyn Error>> {
    let mut bytes = [0u8; CODE_LEN];
    OsRng.fill_bytes(&mut bytes);
    let mut code = String::new();
    let divider = 256 / ALPHABET.len();
    for byte in bytes.into_iter() {
        let i = byte as usize / divider;
        code.push(ALPHABET[i]);
    }
    Ok(code)
}
