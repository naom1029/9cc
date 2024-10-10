use crate::{
    chibicc::{Node, NodeKind},
    error,
};

pub fn gen(node: Box<Node>) {
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

pub fn codegen(node: Box<Node>) {
    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    gen(node);

    // スタックトップに式全体の値を残しているので
    // それをRAXにロードして関数からの戻り値とする
    println!("  pop rax");
    println!("  ret");
}
