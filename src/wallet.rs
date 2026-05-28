use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::RngCore;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct WalletData {
    secret_key: Vec<u8>,
    public_key: Vec<u8>,
}

pub struct Wallet {
    pub signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
}

impl Wallet {
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let mut secret = [0u8; 32];
        csprng.fill_bytes(&mut secret);
        let signing_key = SigningKey::from_bytes(&secret);
        let verifying_key = signing_key.verifying_key();
        let wallet = Wallet {
            signing_key,
            verifying_key,
        };
        wallet.save();
        wallet
    }

    pub fn load() -> Option<Self> {
        let path = Path::new("wallet.json");
        if !path.exists() {
            return None;
        }
        let json = std::fs::read_to_string(path).ok()?;
        let data: WalletData = serde_json::from_str(&json).ok()?;
        let signing_key = SigningKey::from_bytes(&data.secret_key.try_into().ok()?);
        let verifying_key = VerifyingKey::from_bytes(&data.public_key.try_into().ok()?).ok()?;
        Some(Wallet {
            signing_key,
            verifying_key,
        })
    }

    pub fn save(&self) {
        let data = WalletData {
            secret_key: self.signing_key.to_bytes().to_vec(),
            public_key: self.verifying_key.to_bytes().to_vec(),
        };
        if let Ok(json) = serde_json::to_string(&data) {
            let _ = std::fs::write("wallet.json", json);
        }
    }

    pub fn address(&self) -> String {
        hex::encode(self.verifying_key.to_bytes())
    }

    pub fn short_address(&self) -> String {
        let full = self.address();
        full[..16].to_string()
    }

    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        self.signing_key.sign(message).to_bytes().to_vec()
    }

    pub fn verify(pubkey_bytes: &[u8], message: &[u8], signature_bytes: &[u8]) -> bool {
        let Ok(verifying_key) = VerifyingKey::from_bytes(&<[u8; 32]>::try_from(pubkey_bytes).unwrap_or([0u8; 32])) else {
            return false;
        };
        let sig_arr: [u8; 64] = match signature_bytes.try_into() {
            Ok(b) => b,
            Err(_) => return false,
        };
        let signature = Signature::from_bytes(&sig_arr);
        verifying_key.verify(message, &signature).is_ok()
    }
}
