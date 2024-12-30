use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use chrono::Utc;
use rand::{rngs::StdRng, Rng, SeedableRng};

pub fn generate_token() -> String {
    let secret = std::env::var("SECRET").unwrap();
    let seed = {
        let mut seed = [0u8; 32];

        let mut entropy = [0u8; 32];
        StdRng::from_entropy().fill(&mut entropy);
        let timestamp = Utc::now().timestamp().to_ne_bytes();
        let secret = secret.as_bytes();

        for (i, &byte) in secret.iter().enumerate() {
            seed[i % seed.len()] ^= byte;
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
