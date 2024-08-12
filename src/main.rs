use lazy_static::lazy_static;
use std::fmt::Write as FmtWrite;
use std::sync::Mutex;
use std::{fmt, process, str::FromStr};

#[derive(Debug, PartialEq)]
enum NodeKind {
    NdAdd, // +
    NdSub, // -
    NdMul, // *
    NdDiv, // /
    NdNum, // 整数
}

struct Node {
    kind: NodeKind,         // ノードの型
    lhs: Option<Box<Node>>, // 左辺
    rhs: Option<Box<Node>>, // 右辺
    val: Option<i32>,       // kindがNdNumの場合のみ使う
}

#[derive(Debug, PartialEq)]
enum TokenKind {
    TkReserved, // 記号
    TkNum,      // 整数トークン
    TkEof,      // 入力の終わりを表すトークン
}
// トークン型
#[derive(Debug)]
struct Token {
    kind: TokenKind,          // トークンの型
    next: Option<Box<Token>>, // 次の入力トークン
    val: Option<i32>,         // kindがTK_NUMの場合、その数値
    str: String,              // トークン文字列
    pos: usize,               // トークンの位置
}
impl Default for Token {
    fn default() -> Self {
        Token {
            kind: TokenKind::TkEof,
            next: None,
            val: None,
            str: String::new(),
            pos: 0,
        }
    }
}
lazy_static! {
    static ref USER_INPUT: Mutex<String> = Mutex::new(String::new());
}
#[allow(dead_code)]
fn error_at(pos: usize, args: fmt::Arguments) {
    let mut buffer = String::new();

    {
        let user_input = USER_INPUT.lock().unwrap();
        writeln!(buffer, "{}", *user_input).unwrap();
        writeln!(buffer, "{:width$}^ ", "", width = pos - 1).unwrap();
    }

    writeln!(buffer, "{}", args).unwrap();
    eprintln!("{}", buffer);
    process::exit(1);
}

macro_rules! error_at {
    ($loc:expr, $($arg:tt)*) => {
        error_at($loc, format_args!($($arg)*));
    };
}

fn error(args: fmt::Arguments) {
    eprintln!("{}", args);
    process::exit(1);
}

macro_rules! error {
    ($($arg:tt)*) => {
        error(format_args!($($arg)*))
    };
}

fn new_node(kind: NodeKind) -> Box<Node> {
    let node = Box::new(Node {
        kind,
        rhs: None,
        lhs: None,
        val: None,
    });
    return node;
}

fn new_binary(kind: NodeKind, lhs: Box<Node>, rhs: Box<Node>) -> Box<Node> {
    let mut node = new_node(kind);
    node.lhs = Some(lhs);
    node.rhs = Some(rhs);
    return node;
}

fn new_num(val: i32) -> Box<Node> {
    let mut node = new_node(NodeKind::NdNum);
    node.val = Some(val);
    return node;
}

fn expr(token: &mut Option<Box<Token>>) -> Box<Node> {
    let mut node = mul(token);
    loop {
        if consume(token, '+') {
            node = new_binary(NodeKind::NdAdd, node, mul(token));
        } else if consume(token, '-') {
            node = new_binary(NodeKind::NdSub, node, mul(token))
        } else {
            return node;
        }
    }
}

fn mul(token: &mut Option<Box<Token>>) -> Box<Node> {
    let mut node = unary(token);
    loop {
        if consume(token, '*') {
            node = new_binary(NodeKind::NdMul, node, unary(token));
        } else if consume(token, '/') {
            node = new_binary(NodeKind::NdDiv, node, unary(token))
        } else {
            return node;
        }
    }
}
fn unary(token: &mut Option<Box<Token>>) -> Box<Node> {
    if consume(token, '+') {
        return unary(token);
    }
    if consume(token, '-') {
        return new_binary(NodeKind::NdSub, new_num(0), unary(token));
    }
    return primary(token);
}
fn primary(token: &mut Option<Box<Token>>) -> Box<Node> {
    // 次のトークンが"("なら、"("expr")"のはず
    if consume(token, '(') {
        let node = expr(token);
        expect(token, ')');
        return node;
    }
    return new_num(expect_number(token));
}

fn gen(node: Box<Node>) {
    if node.kind == NodeKind::NdNum {
        println!("  push {}", node.val.unwrap());
        return;
    }
    gen(node.lhs.unwrap());
    gen(node.rhs.unwrap());

    println!("  pop rdi");
    println!("  pop rax");

    match node.kind {
        NodeKind::NdAdd => println!("   add rax, rdi"),
        NodeKind::NdSub => println!("   sub rax, rdi"),
        NodeKind::NdMul => println!("   imul rax, rdi"),
        NodeKind::NdDiv => {
            println!("  cqo");
            println!("  idiv rdi");
        }
        _ => error!("不正なnodeです"),
    }
    println!("  push rax");
}

fn consume(token: &mut Option<Box<Token>>, op: char) -> bool {
    if let Some(t) = token {
        if t.kind == TokenKind::TkReserved && t.str.chars().next() == Some(op) {
            *token = t.next.take();
            return true;
        }
    }
    false
}
fn expect(token: &mut Option<Box<Token>>, op: char) {
    if let Some(t) = token {
        if t.kind != TokenKind::TkReserved || t.str.chars().next() != Some(op) {
            error_at!(t.pos, "'{}'ではありません", op);
        }
        *token = t.next.take();
    } else {
        error!("'{}'ではありません", op);
    }
}
fn expect_number(token: &mut Option<Box<Token>>) -> i32 {
    if let Some(t) = token {
        if t.kind != TokenKind::TkNum {
            error_at!(t.pos, "数ではありません");
        }
        let val = t.val.unwrap();
        *token = t.next.take();
        return val;
    } else {
        error!("数ではありません");
        0
    }
}
#[allow(dead_code)]
fn at_eof(token: &Option<Box<Token>>) -> bool {
    if let Some(t) = token {
        t.kind == TokenKind::TkEof
    } else {
        false
    }
}

fn new_token(kind: TokenKind, cur: &mut Box<Token>, str: String, pos: usize) -> &mut Box<Token> {
    let token = Box::new(Token {
        kind,
        next: None,
        val: None,
        str,
        pos,
    });
    cur.next = Some(token);
    // cur.nextをSomeから取り出して返す
    cur.next.as_mut().unwrap()
}
fn tokenize(input: &str) -> Option<Box<Token>> {
    let mut p = input.chars().peekable();
    let mut head = Box::new(Token::default());
    let mut cur = &mut head;
    let mut pos = cur.pos;
    while let Some(&c) = p.peek() {
        pos += 1;
        if c.is_whitespace() {
            p.next();
            continue;
        }
        if c == '+' || c == '-' || c == '*' || c == '/' || c == '(' || c == ')' {
            cur = new_token(TokenKind::TkReserved, cur, c.to_string(), pos);
            p.next();
            continue;
        }

        if c.is_digit(10) {
            let mut num_str = String::new();
            while let Some(&c) = p.peek() {
                if c.is_digit(10) {
                    num_str.push(c);
                    p.next();
                } else {
                    break;
                }
            }
            cur = new_token(TokenKind::TkNum, cur, num_str.clone(), pos);
            cur.val = Some(i32::from_str(&num_str).unwrap());
            continue;
        }

        error_at!(pos, "トークナイズできません");
    }

    new_token(TokenKind::TkEof, cur, String::new(), pos);
    head.next
}

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

    // アセンブリの前半部分を出力
    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    // 抽象構文木を下りながらコード生成
    gen(node);

    // スタックトップに式全体の値が残っているはずなので
    // それをRAXにろーどして関数からの戻り値とする
    println!("  pop rax");
    println!("  ret");
}
// mod tests {
//     use super::*;
//     use std::process::Command;

//     fn create_token_chain(tokens: Vec<(TokenKind, String)>) -> Box<Token> {
//         let mut head: Box<Token> = Box::new(Token::default());
//         let mut cur: &mut Box<Token> = &mut head;

//         for (kind, ch) in tokens {
//             cur = new_token(kind, cur, ch);
//         }
//         head
//     }

//     #[test]
//     fn test_new_token_reserved() {
//         let mut cur = Box::new(Token::default());
//         let token = new_token(TokenKind::TkReserved, &mut cur, '+');
//         assert_eq!(token.kind, TokenKind::TkReserved);
//         assert_eq!(token.str, '+');
//     }

//     #[test]
//     fn test_create_token_chain() {
//         let tokens = vec![
//             (TokenKind::TkReserved, '+'),
//             (TokenKind::TkNum, '1'),
//             (TokenKind::TkEof, '\0'),
//         ];
//         let token_chain = create_token_chain(tokens);

//         // headの次のトークンを検証
//         let mut cur = &token_chain.next;

//         assert_eq!(cur.as_ref().unwrap().kind, TokenKind::TkReserved);
//         assert_eq!(cur.as_ref().unwrap().str, '+');

//         cur = &cur.as_ref().unwrap().next;
//         assert_eq!(cur.as_ref().unwrap().kind, TokenKind::TkNum);
//         assert_eq!(cur.as_ref().unwrap().str, '1');

//         cur = &cur.as_ref().unwrap().next;
//         assert_eq!(cur.as_ref().unwrap().kind, TokenKind::TkEof);
//         assert_eq!(cur.as_ref().unwrap().str, '\0');
//     }
//     #[test]
//     fn test_tokenize() {
//         let input = "1 + 2 - 3";
//         let token_chain = tokenize(input);

//         let expected_tokens = vec![
//             (TokenKind::TkNum, '1', 1),
//             (TokenKind::TkReserved, '+', 0),
//             (TokenKind::TkNum, '2', 2),
//             (TokenKind::TkReserved, '-', 0),
//             (TokenKind::TkNum, '3', 3),
//             (TokenKind::TkEof, '\0', 0),
//         ];

//         let mut cur = &Some(token_chain);
//         for (expected_kind, expected_str, expected_val) in expected_tokens {
//             if let Some(token) = cur {
//                 assert_eq!(token.kind, expected_kind);
//                 assert_eq!(token.str, expected_str);
//                 assert_eq!(token.val, expected_val);
//                 cur = &token.next;
//             } else {
//                 panic!("トークンの数が不足しています");
//             }
//         }
//     }
// }
