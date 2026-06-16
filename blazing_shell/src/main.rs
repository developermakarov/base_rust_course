use std::env;
use std::io::{self, Write};
use std::process::{Command, ExitStatus};

fn check_exit_status(status: ExitStatus) {
    if !status.success() {
        match status.code() {
            Some(code) => eprintln!("Process ended with code:{}", code),
            None => eprintln!(
                "Clippy said, that I must think about None case here))))NO CODE 4 THIS PROCESS"
            ),
        }
    }
}

fn main() {
    loop {
        // вроде бы только две строки...
        // т.к. Rust очень хочет блейзить, то он буферизирует вывод в памяти => как сбросить буфер,
        // чтобы увидеть так называемый prompt??? он сбрасывается, когда:
        // 1.Буфер заполнился
        // 2. \n (btw из-за этого println работает)
        // 3.Программа завершилась(я видел много prompt, когда exit написал)
        // 4.И я явно использую flush (вот это мне надо здесь)
        print!("blazing_shell> ");
        io::stdout().flush().unwrap();
        let mut input_string = String::new();
        match io::stdin().read_line(&mut input_string) {
            Ok(_) => {
                // лучше работать с чистыми строками
                let input_string = input_string.trim();
                let args: Vec<&str> = input_string.split_whitespace().collect();
                if args.is_empty() {
                    continue;
                }
                let command_name = args[0];
                let command_args = &args[1..];
                if command_name == "exit" {
                    println!("Bye Bye, MrTeamlead ^_^");
                    break;
                }
                let mut command = Command::new(command_name);
                command.args(command_args);
                match command.status() {
                    Ok(status) => {
                        check_exit_status(status);
                    }
                    Err(error) => {
                        eprintln!("Error while running'{}': {}", command_name, error);
                        continue;
                    }
                }
            }
            Err(error) => {
                eprintln!("Error while reading input: {}", error);
                break;
            }
        }
    }
}
