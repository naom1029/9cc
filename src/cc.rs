use lazy_static::lazy_static;
use std::sync::Mutex;

#[derive(Debug, PartialEq, Clone)]
pub enum NodeKind {
    NdAdd,    // +
    NdSub,    // -
    NdMul,    // *
    NdDiv,    // /
    NdAssign, // =
    NdLvar,   // ローカル変数
    NdEq,     // ==
    NdNe,     // !=
    NdLt,     // <
    NdLe,     // <=
    NdNum,    // 整数
}
#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,         // ノードの型
    pub lhs: Option<Box<Node>>, // 左辺
    pub rhs: Option<Box<Node>>, // 右辺
    pub val: Option<i32>,       // kindがNdNumの場合のみ使う
    pub offset: Option<i32>,    // kindがNdLvarの場合のみ使う
}

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    TkReserved, // 記号
    TkIndent,   // 識別子
    TkNum,      // 整数トークン
    TkEof,      // 入力の終わりを表すトークン
}
lazy_static! {
    pub static ref CODE: Mutex<Vec<Option<Box<Node>>>> = Mutex::new(Vec::with_capacity(100));
}
// トークン型
#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,          // トークンの型
    pub next: Option<Box<Token>>, // 次の入力トークン
    pub val: Option<i32>,         // kindがTK_NUMの場合、その数値
    pub str: String,              // トークン文字列
    pub pos: usize,               // トークンの位置
    pub len: usize,               // トークンの長さ
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
