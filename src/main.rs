mod blockchain;

use blockchain::Blockchain;
use std::io::{self, Write};

fn main() {
    println!("===================================");
    println!("   Rust 区块链演示");
    println!("   难度: 4 (前导零数量)");
    println!("===================================");

    let mut chain = Blockchain::new();
    println!("\n正在创建创世区块...\n");

    loop {
        println!();
        println!("--- 菜单 ---");
        println!("1. 添加新区块");
        println!("2. 查看整条链");
        println!("3. 验证区块链");
        println!("4. 退出");
        print!("请选择 (1-4): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        match input.trim() {
            "1" => {
                print!("请输入区块数据: ");
                io::stdout().flush().unwrap();
                let mut data = String::new();
                io::stdin().read_line(&mut data).unwrap();
                let data = data.trim().to_string();
                if data.is_empty() {
                    println!("数据不能为空！");
                    continue;
                }
                println!("\n正在挖矿 (寻找有效哈希)...");
                chain.add_block(data);
                let latest = chain.chain.last().unwrap();
                println!("\n新区块已成功添加:");
                println!("{}", latest);
            }
            "2" => {
                println!("\n===== 区块链总览 (共 {} 个区块) =====", chain.chain.len());
                for block in &chain.chain {
                    println!("{}", block);
                }
            }
            "3" => {
                print!("\n正在验证区块链完整性... ");
                io::stdout().flush().unwrap();
                if chain.is_valid() {
                    println!("\x1b[32m有效!\x1b[0m 该区块链未被篡改。");
                } else {
                    println!("\x1b[31m无效!\x1b[0m 该区块链已被篡改！");
                }
            }
            "4" => {
                println!("再见！");
                break;
            }
            _ => println!("无效选项，请重新选择。"),
        }
    }
}
