use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
    pub timestamp: i64,
    pub signature: Vec<u8>,
}

impl Transaction {
    pub fn new(sender: String, receiver: String, amount: u64) -> Self {
        Transaction {
            sender,
            receiver,
            amount,
            timestamp: chrono::Utc::now().timestamp(),
            signature: Vec::new(),
        }
    }

    pub fn coinbase(receiver: String, amount: u64) -> Self {
        Transaction {
            sender: "COINBASE".to_string(),
            receiver,
            amount,
            timestamp: chrono::Utc::now().timestamp(),
            signature: Vec::new(),
        }
    }

    pub fn sign_data(&self) -> Vec<u8> {
        format!(
            "{}:{}:{}:{}",
            self.sender, self.receiver, self.amount, self.timestamp
        )
        .into_bytes()
    }

    pub fn sign(&mut self, wallet: &crate::wallet::Wallet) {
        self.signature = wallet.sign(&self.sign_data());
    }

    pub fn verify(&self) -> bool {
        if self.sender == "COINBASE" {
            return true;
        }
        let Ok(pubkey) = hex::decode(&self.sender) else {
            return false;
        };
        crate::wallet::Wallet::verify(&pubkey, &self.sign_data(), &self.signature)
    }

    pub fn id(&self) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&self.sign_data());
        hasher.update(&self.signature);
        hex::encode(hasher.finalize())
    }
}
