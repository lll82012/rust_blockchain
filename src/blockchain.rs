use crate::transaction::Transaction;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt;

const MIN_DIFFICULTY: usize = 2;
const MAX_DIFFICULTY: usize = 6;
const DIFFICULTY_ADJUST_INTERVAL: u64 = 10;
const TARGET_BLOCK_TIME: i64 = 10;
const INITIAL_MINING_REWARD: u64 = 50;
const REWARD_HALVING_INTERVAL: u64 = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub miner: String,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
    pub difficulty: usize,
}

impl Block {
    pub fn new(
        index: u64,
        transactions: Vec<Transaction>,
        miner: String,
        previous_hash: String,
        difficulty: usize,
    ) -> Self {
        let timestamp = Utc::now().timestamp();
        let mut block = Block {
            index,
            timestamp,
            transactions,
            miner,
            previous_hash,
            hash: String::new(),
            nonce: 0,
            difficulty,
        };
        block.mine();
        block
    }

    fn transactions_hash(&self) -> String {
        let mut hasher = Sha256::new();
        for tx in &self.transactions {
            hasher.update(tx.id().as_bytes());
        }
        hex::encode(hasher.finalize())
    }

    fn calculate_hash(&self) -> String {
        let input = format!(
            "{}{}{}{}{}{}{}",
            self.index,
            self.timestamp,
            self.transactions_hash(),
            self.miner,
            self.previous_hash,
            self.nonce,
            self.difficulty,
        );
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        hex::encode(hasher.finalize())
    }

    fn mine(&mut self) {
        let prefix = "0".repeat(self.difficulty);
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
        let prefix = "0".repeat(self.difficulty);
        if !self.hash.starts_with(&prefix) {
            return false;
        }
        for tx in &self.transactions {
            if !tx.verify() {
                return false;
            }
        }
        self.hash == self.calculate_hash()
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "━━━ 区块 #{} ━━━", self.index)?;
        writeln!(f, "  时间戳:  {}", self.timestamp)?;
        writeln!(f, "  矿工:    {}", &self.miner[..self.miner.len().min(16)])?;
        writeln!(f, "  交易数:  {}", self.transactions.len())?;
        for (i, tx) in self.transactions.iter().enumerate() {
            let from = if tx.sender == "COINBASE" {
                "COINBASE".to_string()
            } else {
                tx.sender[..8].to_string()
            };
            let to = &tx.receiver[..8];
            writeln!(f, "    [{i}] {from} → {to}  {:.8} 币", tx.amount as f64 / 100_000_000.0)?;
        }
        writeln!(f, "  前驱哈希: {}", self.previous_hash)?;
        writeln!(f, "  哈希值:  {}", self.hash)?;
        writeln!(f, "  难度:    {} | 随机数: {}", self.difficulty, self.nonce)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct BlockchainData {
    chain: Vec<Block>,
    difficulty: usize,
    mining_reward: u64,
    balances: HashMap<String, i64>,
}

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub difficulty: usize,
    pub mining_reward: u64,
    pub balances: HashMap<String, i64>,
    save_path: String,
}

impl Blockchain {
    pub fn new(save_path: &str) -> Self {
        let genesis_miner = "GENESIS".to_string();
        let genesis = Block::new(
            0,
            Vec::new(),
            genesis_miner.clone(),
            "0".to_string(),
            MIN_DIFFICULTY,
        );

        Blockchain {
            chain: vec![genesis],
            pending_transactions: Vec::new(),
            difficulty: MIN_DIFFICULTY,
            mining_reward: INITIAL_MINING_REWARD,
            balances: HashMap::new(),
            save_path: save_path.to_string(),
        }
    }

    pub fn load(save_path: &str) -> Option<Self> {
        let json = std::fs::read_to_string(save_path).ok()?;
        let data: BlockchainData = serde_json::from_str(&json).ok()?;
        Some(Blockchain {
            chain: data.chain,
            pending_transactions: Vec::new(),
            difficulty: data.difficulty,
            mining_reward: data.mining_reward,
            balances: data.balances,
            save_path: save_path.to_string(),
        })
    }

    pub fn save(&self) {
        let data = BlockchainData {
            chain: self.chain.clone(),
            difficulty: self.difficulty,
            mining_reward: self.mining_reward,
            balances: self.balances.clone(),
        };
        if let Ok(json) = serde_json::to_string_pretty(&data) {
            let _ = std::fs::write(&self.save_path, json);
        }
    }

    fn adjust_difficulty(&mut self) {
        let len = self.chain.len() as u64;
        if len < 2 || len % DIFFICULTY_ADJUST_INTERVAL != 0 {
            return;
        }
        let recent_start = (len - DIFFICULTY_ADJUST_INTERVAL) as usize;
        let elapsed = self.chain.last().unwrap().timestamp
            - self.chain[recent_start].timestamp;
        let avg_time = elapsed / DIFFICULTY_ADJUST_INTERVAL as i64;

        let old = self.difficulty;
        if avg_time < TARGET_BLOCK_TIME / 2 {
            self.difficulty = (self.difficulty + 1).min(MAX_DIFFICULTY);
        } else if avg_time > TARGET_BLOCK_TIME * 2 {
            self.difficulty = (self.difficulty - 1).max(MIN_DIFFICULTY);
        }
        if self.difficulty != old {
            println!(
                "  难度调整: {} → {} (平均出块时间 {}秒)",
                old, self.difficulty, avg_time
            );
        }
    }

    pub fn current_reward(&self) -> u64 {
        let halvings = self.chain.len() as u64 / REWARD_HALVING_INTERVAL;
        INITIAL_MINING_REWARD >> halvings
    }

    pub fn add_transaction(&mut self, tx: Transaction) -> Result<(), String> {
        if tx.sender == "COINBASE" {
            return Err("不允许手动创建 coinbase 交易".to_string());
        }
        if tx.amount == 0 {
            return Err("转账金额必须大于 0".to_string());
        }
        if !tx.verify() {
            return Err("交易签名无效".to_string());
        }
        let balance = self.balances.get(&tx.sender).copied().unwrap_or(0);
        if balance < tx.amount as i64 {
            return Err(format!(
                "余额不足: 当前 {} 聪, 需要 {} 聪",
                balance, tx.amount
            ));
        }
        // 预留余额
        *self.balances.get_mut(&tx.sender).unwrap() -= tx.amount as i64;
        self.pending_transactions.push(tx);
        Ok(())
    }

    pub fn mine_pending(&mut self, miner_address: &str) -> Block {
        self.adjust_difficulty();

        let reward = self.current_reward();
        let mut block_transactions: Vec<Transaction> = Vec::new();

        let coinbase = Transaction::coinbase(miner_address.to_string(), reward);
        block_transactions.push(coinbase);
        block_transactions.extend(self.pending_transactions.drain(..));

        let index = self.chain.len() as u64;
        let previous_hash = self.chain.last().unwrap().hash.clone();
        let block = Block::new(
            index,
            block_transactions,
            miner_address.to_string(),
            previous_hash,
            self.difficulty,
        );

        // 更新余额
        for tx in &block.transactions {
            *self.balances.entry(tx.receiver.clone()).or_insert(0) += tx.amount as i64;
        }

        self.mining_reward = reward;
        self.chain.push(block.clone());
        self.save();
        block
    }

    pub fn is_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            if !self.chain[i].is_valid(&self.chain[i - 1]) {
                return false;
            }
        }

        // 重新计算余额验证一致性
        let mut computed: HashMap<String, i64> = HashMap::new();
        for block in &self.chain {
            for tx in &block.transactions {
                *computed.entry(tx.receiver.clone()).or_insert(0) += tx.amount as i64;
            }
        }
        computed == self.balances
    }
}
