use std::process;

pub mod chibicc;
pub mod codegen;
pub mod parse;
pub mod tokenize;
use codegen::codegen;
use parse::expr;
use tokenize::{error, tokenize, USER_INPUT};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        error!("'{}'引数の個数が正しくありません。", args.len());
        process::exit(1);
    }
    let user_input_str = &args[1];
    {
        let mut user_input = USER_INPUT.lock().unwrap();
        *user_input = user_input_str.clone();
    }
    // トークナイズしてパースする
    let mut token = tokenize(&args[1]);
    let node = expr(&mut token);

    // 抽象構文木を下りながらコード生成
    codegen(node);
}
