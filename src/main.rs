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
    NdEq,  // ==
    NdNe,  // !=
    NdLt,  // <
    NdLe,  // <=
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
    len: usize,               // トークンの長さ
}
impl Default for Token {
    fn default() -> Self {
        Token {
            kind: TokenKind::TkEof,
            next: None,
            val: None,
            str: String::new(),
            pos: 0,
            len: 0,
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
    return equality(token);
}

fn equality(token: &mut Option<Box<Token>>) -> Box<Node> {
    let mut node = relational(token);
    loop {
        if consume(token, "==".to_string()) {
            node = new_binary(NodeKind::NdEq, node, relational(token));
        } else if consume(token, "!=".to_string()) {
            node = new_binary(NodeKind::NdNe, node, relational(token));
        } else {
            return node;
        }
    }
}

// relational = add ("<" add | "<=" add | ">" add | ">=" add)*
fn relational(token: &mut Option<Box<Token>>) -> Box<Node> {
    let mut node = add(token);

    loop {
        if consume(token, '<'.to_string()) {
            node = new_binary(NodeKind::NdLt, node, add(token));
        } else if consume(token, "<=".to_string()) {
            node = new_binary(NodeKind::NdLe, node, add(token));
        } else if consume(token, '>'.to_string()) {
            node = new_binary(NodeKind::NdLt, add(token), node)
        } else if consume(token, ">=".to_string()) {
            node = new_binary(NodeKind::NdLe, add(token), node)
        } else {
            return node;
        }
    }
}

// add = mul ("+" mul | "-" mul)*
fn add(token: &mut Option<Box<Token>>) -> Box<Node> {
    let mut node = mul(token);
    loop {
        if consume(token, '+'.to_string()) {
            node = new_binary(NodeKind::NdAdd, node, mul(token));
        } else if consume(token, '-'.to_string()) {
            node = new_binary(NodeKind::NdSub, node, mul(token))
        } else {
            return node;
        }
    }
}

// mul = unary("*" unary | "/" unary)*
fn mul(token: &mut Option<Box<Token>>) -> Box<Node> {
    let mut node = unary(token);
    loop {
        if consume(token, '*'.to_string()) {
            node = new_binary(NodeKind::NdMul, node, unary(token));
        } else if consume(token, '/'.to_string()) {
            node = new_binary(NodeKind::NdDiv, node, unary(token))
        } else {
            return node;
        }
    }
}

// unary = ("+" | "-")? unary
fn unary(token: &mut Option<Box<Token>>) -> Box<Node> {
    if consume(token, '+'.to_string()) {
        return unary(token);
    }
    if consume(token, '-'.to_string()) {
        return new_binary(NodeKind::NdSub, new_num(0), unary(token));
    }
    return primary(token);
}

// primary = "(" expr ")" | num
fn primary(token: &mut Option<Box<Token>>) -> Box<Node> {
    // 次のトークンが"("なら、"("expr")"のはず
    if consume(token, '('.to_string()) {
        let node = expr(token);
        expect(token, ')'.to_string());
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
        NodeKind::NdEq => {
            println!("  cmp rax, rdi");
            println!("  sete al");
            println!("  movzb rax, al");
        }
        NodeKind::NdNe => {
            println!("  cmp rax, rdi");
            println!("  setne al");
            println!("  movzb rax, al");
        }
        NodeKind::NdLt => {
            println!("  cmp rax, rdi");
            println!("  setl al");
            println!("  movzb rax, al");
        }
        NodeKind::NdLe => {
            println!("  cmp rax, rdi");
            println!("  setle al");
            println!("  movzb rax, al");
        }
        _ => error!("不正なnodeです"),
    }
    println!("  push rax");
}

fn consume(token: &mut Option<Box<Token>>, op: String) -> bool {
    if let Some(t) = token {
        if t.kind == TokenKind::TkReserved && op.len() == t.len && op == t.str {
            *token = t.next.take();
            return true;
        }
    }
    false
}
fn expect(token: &mut Option<Box<Token>>, op: String) {
    if let Some(t) = token {
        if t.kind != TokenKind::TkReserved || op.len() != t.len || op != t.str {
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

fn new_token(
    kind: TokenKind,
    cur: &mut Box<Token>,
    str: String,
    pos: usize,
    len: usize,
) -> &mut Box<Token> {
    let token = Box::new(Token {
        kind,
        next: None,
        val: None,
        str,
        pos,
        len,
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

        // 空白の処理
        if c.is_whitespace() {
            p.next();
            continue;
        }

        // 2文字のトークンの処理
        if let Some(next_c) = p.clone().nth(1) {
            let two_char_str = format!("{}{}", c, next_c);
            if two_char_str == "=="
                || two_char_str == "!="
                || two_char_str == "<="
                || two_char_str == ">="
            {
                cur = new_token(TokenKind::TkReserved, cur, two_char_str.clone(), pos, 2);
                p.next(); // 1文字目を進める
                p.next(); // 2文字目を進める
                continue;
            }
        }

        // 1文字のトークンの処理
        if c == '+'
            || c == '-'
            || c == '*'
            || c == '/'
            || c == '('
            || c == ')'
            || c == '<'
            || c == '>'
        {
            cur = new_token(TokenKind::TkReserved, cur, c.to_string(), pos, 1);
            p.next();
            continue;
        }
        // 数値の処理
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
            cur = new_token(TokenKind::TkNum, cur, num_str.clone(), pos, 0);
            cur.val = Some(i32::from_str(&num_str).unwrap());
            cur.len = num_str.len();
            continue;
        }

        error_at!(pos, "トークナイズできません");
    }

    new_token(TokenKind::TkEof, cur, String::new(), pos, 0);
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
    // それをRAXにロードして関数からの戻り値とする
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
