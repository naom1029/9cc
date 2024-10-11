use crate::{
    cc::{Node, NodeKind, CODE},
    error,
};

pub fn gen_lval(node: Box<Node>) {
    if node.kind != NodeKind::NdLvar {
        error!("代入の左辺値が変数ではありません");
    }
    println!("  mov rax, rbp");
    println!("  sub rax, {}", node.offset.unwrap());
    println!("  push rax");
}

pub fn gen(node: Box<Node>) {
    if node.kind == NodeKind::NdNum {
        println!("  push {}", node.val.unwrap());
        return;
    }
    if let Some(ref lhs) = node.lhs {
        gen(lhs.clone());
    }

    if let Some(ref rhs) = node.rhs {
        gen(rhs.clone());
    }

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
        NodeKind::NdLvar => {
            gen_lval(node);
            println!("  pop rax");
            println!("  mov rax, [rax]");
            println!("  push rax");
            return;
        }
        NodeKind::NdAssign => {
            gen_lval(node.lhs.unwrap());
            gen(node.rhs.unwrap());
            println!("  pop rdi");
            println!("  mov [rax], rdi");
            println!("  push rdi");
            return;
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

pub fn codegen() {
    // アセンブリの前半部分を出力
    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    // プロローグ
    //  変数26個分の領域を確保する
    println!("  push rbp");
    println!("  mov rbp, rsp");
    println!("  sub rsp, 208");

    let code = CODE.lock().unwrap();

    for node_option in code.iter() {
        if let Some(node) = node_option {
            gen(node.clone());
            println!("  pop rax");
        }
    }
    // スタックトップに式全体の値を残しているので
    // それをRAXにロードして関数からの戻り値とする
    println!("  pop rax");
    println!("  ret");
}
