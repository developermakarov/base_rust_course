use std::io::{self, Error, Write};
use std::process::{Child, Command, ExitStatus, Stdio};

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

fn run_single_command(args: &[&str]) -> Result<ExitStatus, Error> {
    if args.is_empty() {
        return Err(Error::new(
            std::io::ErrorKind::InvalidInput,
            "empty command",
        ));
    }
    let command_name = args[0];
    let command_args = &args[1..];
    let mut command = Command::new(command_name);
    command.args(command_args);
    command.status()
}

fn run_command_with_stdin(
    args: &[&str],
    stdin_source: impl Into<std::process::Stdio>,
) -> Result<Child, Error> {
    if args.is_empty() {
        return Err(Error::new(
            std::io::ErrorKind::InvalidInput,
            "empty command",
        ));
    }
    let command_name = args[0];
    let command_args = &args[1..];
    let mut command = Command::new(command_name);
    command.args(command_args);
    command.stdin(stdin_source);
    command.stdout(Stdio::inherit());
    command.stderr(Stdio::inherit());
    command.spawn()
}
fn run_pipe(left_args: &[&str], right_args: &[&str]) {
    /*
    чтобы реализовать pipe нужно использовать Stdio::piped() для output левой команды,
    и передаём вывод ребёнка как input для правой команды
    =================================================================================
    по реализации:
    1. проверка аргументов
    2. проверка на exit
    3. готовим левую команду
    4. готовим output для левой команды
    тут мы говорим ОС, что когда запустится процесс не отдавай его вывод терминалу(это output родителя)
    а создай мне pipe и туда вывод отдай. вот тут вопрос тоже такой, по идее мы конфигурируем поведение pipe
    с помощью Stdio::piped() а сам pipe создаётся при спавне ребенка ???? правильно ???
    5. запуск левой команды через спавн ребёнка
    6. тут огромная духота с владением , чтобы получить значение ChildStdout, в питоне такого нет
    я чуть не помер тут, если кратко, то поспользуемся take чтобы забрать значение output у ребенка, дать ему None
    и записать значение в left_stdout , чтобы отдать на вход правой команде
    7. запускаем правую команду по связанному pipe
    8. ждём завершения работы процессов, сперва правый , потому что он читает из pipe.
    wait залочит выполнение run_pipe пока дочерние процессы не завершат работу. ну и всё
    9. чек статусов
    */
    if left_args.is_empty() || right_args.is_empty() {
        eprintln!("empty command inside pipe");
        return;
    }
    let left_command_name = left_args[0];
    let right_command_name = right_args[0];
    // ну вот так вот )))
    if left_command_name == "exit" || right_command_name == "exit" {
        eprintln!("pipe doesn't support 'exit' command, sry");
        return;
    }

    let mut left_command = Command::new(left_command_name);
    left_command.args(&left_args[1..]);
    left_command.stdout(Stdio::piped());
    left_command.stderr(Stdio::inherit());

    let mut left_child = match left_command.spawn() {
        Ok(child) => child,
        Err(error) => {
            eprintln!("Error while running'{}': {}", left_command_name, error);
            return;
        }
    };

    let left_stdout = match left_child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            eprintln!("can't get stdout from left command");
            let _ = left_child.kill();
            let _ = left_child.wait();
            return;
        }
    };

    let mut right_child = match run_command_with_stdin(right_args, left_stdout) {
        Ok(child) => child,
        Err(error) => {
            eprintln!("error while running '{}': {}", right_command_name, error);
            let _ = left_child.kill();
            let _ = left_child.wait();
            return;
        }
    };

    let right_status = match right_child.wait() {
        Ok(status) => status,
        Err(error) => {
            eprintln!(
                "error, w8ing right command '{}': {}",
                right_command_name, error
            );
            let _ = left_child.wait();
            return;
        }
    };

    let left_status = match left_child.wait() {
        Ok(status) => status,
        Err(error) => {
            eprintln!(
                "error, w8ing left command'{}': {}",
                left_command_name, error
            );
            check_exit_status(right_status);
            return;
        }
    };

    check_exit_status(left_status);
    check_exit_status(right_status);
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
                let pipe_parts: Vec<&str> = input_string.split('|').map(|s| s.trim()).collect();
                if pipe_parts.len() == 2 {
                    // Обнаружен один pipe. Запускаем piped execution.
                    let left_cmd_str = pipe_parts[0];
                    let right_cmd_str = pipe_parts[1];

                    if left_cmd_str.is_empty() || right_cmd_str.is_empty() {
                        eprintln!("empty command inside pipe: '{}'", input_string);
                        continue;
                    }

                    let left_args: Vec<&str> = left_cmd_str.split_whitespace().collect();
                    let right_args: Vec<&str> = right_cmd_str.split_whitespace().collect();

                    run_pipe(&left_args, &right_args);
                    continue;
                } else if pipe_parts.len() > 2 {
                    eprintln!(
                        "unsupported pipe format:'{}'.We work only with 2 commands in pipe",
                        input_string
                    );
                    continue;
                }
                let args: Vec<&str> = input_string.split_whitespace().collect();
                if args.is_empty() {
                    continue;
                }
                let command_name = args[0];
                if command_name == "exit" {
                    println!("Bye Bye, MrTeamlead ^_^");
                    break;
                }
                match run_single_command(&args) {
                    Ok(status) => {
                        check_exit_status(status);
                    }
                    Err(error) => {
                        eprintln!("error while running '{}': {}", command_name, error);
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
