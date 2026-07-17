use std::io::{self, Write};

use game::{DEFAULT_WORD, GuessResult, Hangman, MAX_ATTEMPTS};

fn main() {
    let mut game = Hangman::new(DEFAULT_WORD, MAX_ATTEMPTS);
    let stdin = io::stdin();

    println!("Добро пожаловать в игру «Виселица»!");
    println!();

    loop {
        print_status(&game);

        if game.is_won() || game.is_lost() {
            break;
        }

        print!("Введите букву: ");
        if let Err(error) = io::stdout().flush() {
            eprintln!("Ошибка вывода: {error}");
            break;
        }

        let mut input = String::new();
        match stdin.read_line(&mut input) {
            Ok(0) => {
                println!("Ввод завершён.");
                break;
            }
            Ok(_) => {}
            Err(error) => {
                eprintln!("Ошибка чтения ввода: {error}");
                break;
            }
        }

        match game.guess(&input) {
            GuessResult::Invalid => {
                println!("Некорректный ввод: введите ровно одну букву.");
            }
            GuessResult::AlreadyGuessed => {
                println!("Эта буква уже была введена.");
            }
            GuessResult::Hit => {
                println!("Есть такая буква!");
            }
            GuessResult::Miss => {
                println!("Такой буквы нет.");
            }
            GuessResult::Won => {
                println!("Есть такая буква!");
            }
            GuessResult::Lost => {
                println!("Такой буквы нет.");
            }
        }

        println!();
    }

    println!();

    if game.is_won() {
        println!("Победа! Вы угадали слово: {}", game.secret_word());
    } else if game.is_lost() {
        println!("Поражение. Правильное слово: {}", game.secret_word());
    } else {
        println!("Игра прервана. Правильное слово: {}", game.secret_word());
    }
}

fn print_status(game: &Hangman) {
    println!("Слово: {}", game.masked_word());

    let guessed = game.guessed_letters();
    if guessed.is_empty() {
        println!("Введённые буквы: —");
    } else {
        println!("Введённые буквы: {guessed}");
    }

    println!("Осталось попыток: {}", game.attempts_left());
}
