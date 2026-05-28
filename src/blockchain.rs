use chrono::Utc;
use sha2::{Digest, Sha256};
use std::fmt;

const DIFFICULTY: usize = 4;

#[derive(Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub data: String,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
}

impl Block {
    pub fn new(index: u64, data: String, previous_hash: String) -> Self {
        let timestamp = Utc::now().timestamp();
        let mut block = Block {
            index,
            timestamp,
            data,
            previous_hash,
            hash: String::new(),
            nonce: 0,
        };
        block.mine();
        block
    }

    fn calculate_hash(&self) -> String {
        let input = format!(
            "{}{}{}{}{}",
            self.index, self.timestamp, self.data, self.previous_hash, self.nonce
        );
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        hex::encode(hasher.finalize())
    }

    fn mine(&mut self) {
        let prefix = "0".repeat(DIFFICULTY);
        loop {
            self.hash = self.calculate_hash();
            if self.hash.starts_with(&prefix) {
                break;
            }
            self.nonce += 1;
        }
    }

    pub fn is_valid(&self, previous_block: &Block) -> bool {
        if self.index != previous_block.index + 1 {
            return false;
        }
        if self.previous_hash != previous_block.hash {
            return false;
        }
        let prefix = "0".repeat(DIFFICULTY);
        if !self.hash.starts_with(&prefix) {
            return false;
        }
        self.hash == self.calculate_hash()
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "  区块 #{}", self.index)?;
        writeln!(f, "  时间戳:  {}", self.timestamp)?;
        writeln!(f, "  数据:    {}", self.data)?;
        writeln!(f, "  前驱哈希: {}", self.previous_hash)?;
        writeln!(f, "  哈希值:  {}", self.hash)?;
        writeln!(f, "  随机数:  {}", self.nonce)?;
        Ok(())
    }
}

pub struct Blockchain {
    pub chain: Vec<Block>,
}

impl Blockchain {
    pub fn new() -> Self {
        let genesis = Block::new(0, "创世区块".to_string(), "0".to_string());
        Blockchain {
            chain: vec![genesis],
        }
    }

    pub fn add_block(&mut self, data: String) {
        let previous_hash = self.chain.last().unwrap().hash.clone();
        let index = self.chain.len() as u64;
        let block = Block::new(index, data, previous_hash);
        self.chain.push(block);
    }

    pub fn is_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current = &self.chain[i];
            let previous = &self.chain[i - 1];
            if !current.is_valid(previous) {
                return false;
            }
        }
        true
    }
}
