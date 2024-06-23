fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("引数の個数が正しくありません");
        std::process::exit(1);
    }
    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");
    println!("  mov rax, {}", args[1]);
    println!("  ret");
    std::process::exit(0);
}

mod tests {
    // use super::*;
    use std::process::Command;

    #[test]
    fn test_main123() {
        let output = Command::new("cargo")
            .arg("run")
            .arg("--")
            .arg("123")
            .output()
            .expect("failed to execute process");

        assert!(output.status.success());
        let stdout_str = String::from_utf8_lossy(&output.stdout);
        assert!(stdout_str.contains("mov rax, 123"));
    }
    #[test]
    fn test_main42() {
        let output = Command::new("cargo")
            .arg("run")
            .arg("--")
            .arg("42")
            .output()
            .expect("failed to execute process");

        assert!(output.status.success());
        let stdout_str = String::from_utf8_lossy(&output.stdout);
        assert!(stdout_str.contains("mov rax, 42"));
    }
}
