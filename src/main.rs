use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect(); // コマンドライン引数をベクターに収集

    if args.len() != 2 {
        eprintln!("引数の個数が正しくありません");
        process::exit(1); // プログラムを終了
    }

    let p = args[1].as_str(); // コマンドライン引数をスライスとして取得

    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    // 最初の数値を取得して表示
    let (mut value, mut p) = match parse_number(p) {
        Some(result) => result,
        None => {
            eprintln!("数値の変換に失敗しました");
            process::exit(1);
        }
    };
    println!("  mov rax, {}", value);

    while !p.is_empty() {
        match p.chars().next() {
            Some('+') => {
                p = &p[1..];
                match parse_number(p) {
                    Some((num, rest)) => {
                        value = num;
                        p = rest;
                        println!("  add rax, {}", value);
                    }
                    None => {
                        eprintln!("数値の変換に失敗しました");
                        process::exit(1);
                    }
                }
            }
            Some('-') => {
                p = &p[1..];
                match parse_number(p) {
                    Some((num, rest)) => {
                        value = num;
                        p = rest;
                        println!("  sub rax, {}", value);
                    }
                    None => {
                        eprintln!("数値の変換に失敗しました");
                        process::exit(1);
                    }
                }
            }
            Some(c) => {
                eprintln!("予期しない文字です: '{}'", c);
                process::exit(1);
            }
            None => break,
        }
    }

    println!("  ret");
}

fn parse_number(input: &str) -> Option<(i64, &str)> {
    let mut chars = input.chars(); // 文字列のイテレータを作成
    let mut num_str = String::new(); // 数値を保持する文字列を初期化

    while let Some(c) = chars.next() {
        // イテレータから次の文字を取得
        if c.is_digit(10) {
            // 文字が数字であれば
            num_str.push(c); // 数字をnum_strに追加
        } else {
            let remaining = &input[num_str.len()..]; // 残りの文字列を取得
            return num_str.parse::<i64>().ok().map(|num| (num, remaining)); // 数値に変換し、タプルを返す
        }
    }

    num_str.parse::<i64>().ok().map(|num| (num, "")) // 最後まで数字の場合、数値と空文字列を返す
}
// mod tests {
//     // use super::*;
//     use std::process::Command;

//     #[test]
//     fn test_main123() {
//         let output = Command::new("cargo")
//             .arg("run")
//             .arg("--")
//             .arg("123")
//             .output()
//             .expect("failed to execute process");

//         assert!(output.status.success());
//         let stdout_str = String::from_utf8_lossy(&output.stdout);
//         assert!(stdout_str.contains("mov rax, 123"));
//     }
//     #[test]
//     fn test_main42() {
//         let output = Command::new("cargo")
//             .arg("run")
//             .arg("--")
//             .arg("42")
//             .output()
//             .expect("failed to execute process");

//         assert!(output.status.success());
//         let stdout_str = String::from_utf8_lossy(&output.stdout);
//         assert!(stdout_str.contains("mov rax, 42"));
//     }
// }
