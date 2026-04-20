use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use chrono::Utc;
use rand::{rngs::StdRng, Rng, SeedableRng};
use sha2::{Digest, Sha512};

pub fn hash_token(token: &str) -> String {
    let hashed_token = Sha512::digest(token.as_bytes());
    BASE64_URL_SAFE_NO_PAD.encode(hashed_token)
}

pub fn generate_token() -> String {
    let secret = match std::env::var("SECRET") {
        Ok(s) => Some(s),
        Err(_) => None,
    };
    let seed = {
        let mut seed = [0u8; 32];

        let mut entropy = [0u8; 32];
        StdRng::from_entropy().fill(&mut entropy);
        let timestamp = Utc::now().timestamp().to_ne_bytes();

        if let Some(s) = secret {
            let secret = s.as_bytes();
            for (i, &byte) in secret.iter().enumerate() {
                seed[i % seed.len()] ^= byte;
            }
        }

        for (i, &byte) in entropy.iter().enumerate() {
            seed[i % seed.len()] ^= byte;
        }
        for (i, &byte) in timestamp.iter().enumerate() {
            for offset in 0..(seed.len() / timestamp.len()) {
                seed[(offset * timestamp.len()) + (i % seed.len())] ^= byte;
            }
        }

        seed
    };
    let mut rng = StdRng::from_seed(seed);
    let mut bytes = [0u8; 32];
    rng.fill(&mut bytes);

    BASE64_URL_SAFE_NO_PAD.encode(bytes)
}
