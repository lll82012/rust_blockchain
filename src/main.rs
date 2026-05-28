mod blockchain;
mod transaction;
mod wallet;

use blockchain::Blockchain;
use transaction::Transaction;
use wallet::Wallet;
use std::io::{self, Write};

const SAVE_PATH: &str = "blockchain.json";

fn main() {
    println!("╔══════════════════════════════════════╗");
    println!("║       Rust 简易加密货币              ║");
    println!("║       基于工作量证明 (PoW)          ║");
    println!("╚══════════════════════════════════════╝");

    let mut chain = match Blockchain::load(SAVE_PATH) {
        Some(c) => {
            println!("\n已从磁盘加载区块链 ({} 个区块)", c.chain.len());
            c
        }
        None => {
            println!("\n未找到已保存的区块链，创建新链...");
            let c = Blockchain::new(SAVE_PATH);
            c.save();
            c
        }
    };

    let mut wallet = Wallet::load();

    loop {
        println!();
        print_status(&chain, &wallet);
        println!();
        println!("--- 菜单 ---");
        println!("1. 创建/加载钱包");
        println!("2. 挖矿 (处理待定交易)");
        println!("3. 转账");
        println!("4. 查询余额");
        println!("5. 查看区块链");
        println!("6. 查看待处理交易");
        println!("7. 验证区块链");
        println!("8. 退出");
        print!("请选择 (1-8): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        match input.trim() {
            "1" => {
                if wallet.is_some() {
                    println!("已有钱包: {}", wallet.as_ref().unwrap().short_address());
                    print!("创建新钱包将覆盖现有钱包，确认? (y/N): ");
                    io::stdout().flush().unwrap();
                    let mut confirm = String::new();
                    io::stdin().read_line(&mut confirm).unwrap();
                    if confirm.trim().to_lowercase() != "y" {
                        continue;
                    }
                }
                wallet = Some(Wallet::generate());
                let w = wallet.as_ref().unwrap();
                println!("\n新钱包已创建并保存到 wallet.json");
                println!("  地址: {}", w.address());
            }
            "2" => {
                let w = match &wallet {
                    Some(w) => w,
                    None => {
                        println!("请先创建或加载钱包 (菜单选项 1)");
                        continue;
                    }
                };
                let miner_addr = w.address();
                println!("\n⛏  开始挖矿...");
                println!("  矿工:    {}", w.short_address());
                println!(
                    "  待处理交易: {} 笔",
                    chain.pending_transactions.len()
                );
                println!(
                    "  挖矿奖励: {} 币",
                    chain.current_reward() as f64 / 100_000_000.0
                );
                println!(
                    "  当前难度: {} (需要 {} 个前导零)",
                    chain.difficulty, chain.difficulty
                );
                println!("  正在计算有效哈希...\n");

                let start = std::time::Instant::now();
                let block = chain.mine_pending(&miner_addr);
                let elapsed = start.elapsed().as_secs_f64();

                println!("\n新区块已打包:");
                println!("{}", block);
                println!(
                    "  挖矿耗时: {:.1}秒 | 哈希率: {:.0} H/s",
                    elapsed,
                    block.nonce as f64 / elapsed.max(0.01)
                );
                let bal = chain.balances.get(&miner_addr).copied().unwrap_or(0);
                println!(
                    "  你的余额: {} 币 ({} 聪)",
                    bal as f64 / 100_000_000.0,
                    bal
                );
            }
            "3" => {
                let w = match &wallet {
                    Some(w) => w,
                    None => {
                        println!("请先创建或加载钱包 (菜单选项 1)");
                        continue;
                    }
                };
                let my_balance = chain
                    .balances
                    .get(&w.address())
                    .copied()
                    .unwrap_or(0);
                if my_balance <= 0 {
                    println!("你的余额为零，无法转账。请先挖矿获取奖励。");
                    continue;
                }
                println!(
                    "你的余额: {} 币 ({} 聪)",
                    my_balance as f64 / 100_000_000.0,
                    my_balance
                );
                print!("请输入接收方地址 (或前16位简写): ");
                io::stdout().flush().unwrap();
                let mut receiver = String::new();
                io::stdin().read_line(&mut receiver).unwrap();
                let receiver = receiver.trim().to_string();
                if receiver.is_empty() {
                    continue;
                }
                print!("请输入转账金额 (聪, 1币 = 100000000 聪): ");
                io::stdout().flush().unwrap();
                let mut amount_str = String::new();
                io::stdin().read_line(&mut amount_str).unwrap();
                let amount: u64 = match amount_str.trim().parse() {
                    Ok(a) if a > 0 => a,
                    _ => {
                        println!("无效金额");
                        continue;
                    }
                };

                let mut tx = Transaction::new(w.address(), receiver.clone(), amount);
                tx.sign(w);
                match chain.add_transaction(tx) {
                    Ok(()) => {
                        println!("\n交易已创建并加入待处理池!");
                        println!(
                            "  发送方: {} → 接收方: {}...",
                            w.short_address(),
                            &receiver[..receiver.len().min(16)]
                        );
                        println!(
                            "  金额: {} 币 ({} 聪)",
                            amount as f64 / 100_000_000.0,
                            amount
                        );
                        println!("  等待矿工打包上链...");
                    }
                    Err(e) => println!("交易失败: {}", e),
                }
            }
            "4" => {
                print!("请输入要查询的地址 (留空则查询自己): ");
                io::stdout().flush().unwrap();
                let mut addr = String::new();
                io::stdin().read_line(&mut addr).unwrap();
                let addr = addr.trim().to_string();

                if addr.is_empty() {
                    if let Some(w) = &wallet {
                        let bal = chain.balances.get(&w.address()).copied().unwrap_or(0);
                        println!("\n你的钱包:");
                        println!("  地址: {}", w.address());
                        println!(
                            "  余额: {} 币 ({} 聪)",
                            bal as f64 / 100_000_000.0,
                            bal
                        );
                    } else {
                        println!("请先创建钱包，或输入要查询的地址");
                    }
                } else {
                    // 支持短地址匹配
                    let found = chain
                        .balances
                        .iter()
                        .find(|(k, _)| k.starts_with(&addr) || **k == addr);
                    match found {
                        Some((address, balance)) => {
                            println!("\n地址: {}", address);
                            println!(
                                "余额: {} 币 ({} 聪)",
                                *balance as f64 / 100_000_000.0,
                                balance
                            );
                        }
                        None => println!("未找到该地址的交易记录"),
                    }
                }
            }
            "5" => {
                println!(
                    "\n===== 区块链总览 (共 {} 个区块) =====",
                    chain.chain.len()
                );
                let total_tx: usize = chain.chain.iter().map(|b| b.transactions.len()).sum();
                println!(
                    "  难度: {} | 奖励: {} 币 | 总交易数: {}",
                    chain.difficulty,
                    chain.mining_reward as f64 / 100_000_000.0,
                    total_tx
                );
                println!();
                // 默认显示最近 5 个区块
                let start = if chain.chain.len() > 5 {
                    chain.chain.len() - 5
                } else {
                    0
                };
                if start > 0 {
                    println!("  ... (省略 {} 个早期区块) ...\n", start);
                }
                for block in &chain.chain[start..] {
                    println!("{}", block);
                }
            }
            "6" => {
                if chain.pending_transactions.is_empty() {
                    println!("\n暂无待处理交易");
                } else {
                    println!(
                        "\n===== 待处理交易 (共 {} 笔) =====",
                        chain.pending_transactions.len()
                    );
                    for (i, tx) in chain.pending_transactions.iter().enumerate() {
                        let from = &tx.sender[..tx.sender.len().min(16)];
                        let to = &tx.receiver[..tx.receiver.len().min(16)];
                        println!(
                            "  [{}] {} → {}  {} 聪 | ID: {}...",
                            i,
                            from,
                            to,
                            tx.amount,
                            &tx.id()[..16]
                        );
                    }
                }
            }
            "7" => {
                print!("\n正在验证区块链完整性... ");
                io::stdout().flush().unwrap();
                if chain.is_valid() {
                    println!("\x1b[32m有效!\x1b[0m 该区块链完整且未被篡改。");
                } else {
                    println!("\x1b[31m无效!\x1b[0m 该区块链已被篡改！");
                }
            }
            "8" => {
                println!("再见！");
                break;
            }
            _ => println!("无效选项，请重新选择。"),
        }
    }
}

fn print_status(chain: &Blockchain, wallet: &Option<Wallet>) {
    let pending = chain.pending_transactions.len();
    let blocks = chain.chain.len();
    let difficulty = chain.difficulty;

    print!(
        "\x1b[36m[链高度: {} | 难度: {} | 待处理: {} 笔",
        blocks, difficulty, pending
    );

    if let Some(w) = wallet {
        let bal = chain.balances.get(&w.address()).copied().unwrap_or(0);
        print!(
            " | 余额: {} 币",
            bal as f64 / 100_000_000.0
        );
    }
    println!("\x1b[0m");
}
