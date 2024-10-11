use crate::cc::{Node, NodeKind, Token, TokenKind, CODE};
use crate::tokenize::{expect, expect_number};

// 次のトークンが期待している記号の時には、トークンを1つ読み進めて真を返す
pub fn consume(token: &mut Option<Box<Token>>, op: String) -> bool {
    if let Some(t) = token {
        if t.kind == TokenKind::TkReserved && op.len() == t.len && op == t.str {
            *token = t.next.take();
            return true;
        }
    }
    false
}
pub fn consume_indent(token: &mut Option<Box<Token>>) -> Option<Box<Token>> {
    if let Some(ref t) = token {
        if t.kind != TokenKind::TkIndent {
            return None;
        }
    }
    token.take()
}
pub fn new_node(kind: NodeKind) -> Box<Node> {
    let node = Box::new(Node {
        kind,
        rhs: None,
        lhs: None,
        val: None,
        offset: None,
    });
    return node;
}

pub fn new_node_lvar(op: &str) -> Box<Node> {
    let mut node = new_node(NodeKind::NdLvar);
    if let Some(first_char) = op.chars().next() {
        node.offset = Some(first_char as i32 - 'a' as i32);
    } else {
        node.offset = None;
    }
    node
}

pub fn new_binary(kind: NodeKind, lhs: Box<Node>, rhs: Box<Node>) -> Box<Node> {
    let mut node = new_node(kind);
    node.lhs = Some(lhs);
    node.rhs = Some(rhs);
    return node;
}

pub fn new_num(val: i32) -> Box<Node> {
    let mut node = new_node(NodeKind::NdNum);
    node.val = Some(val);
    return node;
}

#[allow(dead_code)]
pub fn at_eof(token: &Option<Box<Token>>) -> bool {
    if let Some(t) = token {
        t.kind == TokenKind::TkEof
    } else {
        false
    }
}
// assign = equality ("=" assign)?
pub fn assign(token: &mut Option<Box<Token>>) -> Box<Node> {
    let mut node = equality(token);
    if consume(token, "=".to_string()) {
        node = new_binary(NodeKind::NdAssign, node, assign(token));
    }
    return node;
}
// expr = assign
pub fn expr(token: &mut Option<Box<Token>>) -> Box<Node> {
    return assign(token);
}
// stmt = expr ";"
pub fn stmt(token: &mut Option<Box<Token>>) -> Box<Node> {
    let node = expr(token);
    expect(token, ";".to_string());
    return node;
}
// program = stmt*
pub fn program(token: &mut Option<Box<Token>>) {
    let mut code = CODE.lock().unwrap();

    while !at_eof(token) {
        code.push(Some(stmt(token)));
    }

    // 終端を示すために `None` を追加
    code.push(None);
}
// equality = relational ("==" relational | "!=" relational)*
pub fn equality(token: &mut Option<Box<Token>>) -> Box<Node> {
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
pub fn relational(token: &mut Option<Box<Token>>) -> Box<Node> {
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
pub fn add(token: &mut Option<Box<Token>>) -> Box<Node> {
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
pub fn mul(token: &mut Option<Box<Token>>) -> Box<Node> {
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
pub fn unary(token: &mut Option<Box<Token>>) -> Box<Node> {
    if consume(token, '+'.to_string()) {
        return unary(token);
    }
    if consume(token, '-'.to_string()) {
        return new_binary(NodeKind::NdSub, new_num(0), unary(token));
    }
    return primary(token);
}

// primary = num | ident | "(" expr ")"
pub fn primary(token: &mut Option<Box<Token>>) -> Box<Node> {
    // 次のトークンが"("なら、"("expr")"のはず
    if consume(token, '('.to_string()) {
        let node = expr(token);
        expect(token, ')'.to_string());
        return node;
    }
    return new_num(expect_number(token));
}
