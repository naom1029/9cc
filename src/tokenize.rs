use crate::cc::{Token, TokenKind};
use lazy_static::lazy_static;
use std::fmt::Write as FmtWrite;
use std::str::FromStr;
use std::sync::Mutex;
use std::{fmt, process};

lazy_static! {
    pub static ref USER_INPUT: Mutex<String> = Mutex::new(String::new());
}

pub fn error_at(pos: usize, args: fmt::Arguments) {
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

pub fn error(args: fmt::Arguments) {
    eprintln!("{}", args);
    process::exit(1);
}
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        error(format_args!($($arg)*))
    };
}

pub fn expect(token: &mut Option<Box<Token>>, op: String) {
    if let Some(t) = token {
        if t.kind != TokenKind::TkReserved || op.len() != t.len || op != t.str {
            error_at!(t.pos, "'{}'ではありません", op);
        }
        *token = t.next.take();
    } else {
        error!("'{}'ではありません", op);
    }
}
pub fn expect_number(token: &mut Option<Box<Token>>) -> i32 {
    if let Some(t) = token {
        if t.kind != TokenKind::TkNum {
            error_at!(t.pos, "数ではありません");
        }
        if let Some(val) = t.val {
            *token = t.next.take();
            return val;
        } else {
            error_at!(t.pos, "数の値が見つかりません");
        }
    } else {
        error!("数ではありません");
    }
    panic!("期待される数が見つかりませんでした");
}

pub fn new_token(
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

pub fn tokenize(input: &str) -> Option<Box<Token>> {
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
        // 識別子の処理
        if c.is_alphabetic() {
            cur = new_token(TokenKind::TkIndent, cur, c.to_string(), pos, 1);
        }
        error_at!(pos, "トークナイズできません");
    }

    new_token(TokenKind::TkEof, cur, String::new(), pos, 0);
    head.next
}
