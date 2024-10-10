use crate::chibicc::{Node, NodeKind, Token};
use crate::tokenize::{consume, expect, expect_number};

pub fn new_node(kind: NodeKind) -> Box<Node> {
    let node = Box::new(Node {
        kind,
        rhs: None,
        lhs: None,
        val: None,
    });
    return node;
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

pub fn expr(token: &mut Option<Box<Token>>) -> Box<Node> {
    return equality(token);
}

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

// primary = "(" expr ")" | num
pub fn primary(token: &mut Option<Box<Token>>) -> Box<Node> {
    // 次のトークンが"("なら、"("expr")"のはず
    if consume(token, '('.to_string()) {
        let node = expr(token);
        expect(token, ')'.to_string());
        return node;
    }
    return new_num(expect_number(token));
}
