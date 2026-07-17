use game::{GuessResult, Hangman};

#[test]
fn hit_reveals_all_occurrences() {
    let mut game = Hangman::new("test", 5);

    assert_eq!(game.guess("t"), GuessResult::Hit);
    assert_eq!(game.masked_word(), "t _ _ t");
    assert_eq!(game.attempts_left(), 5);
}

#[test]
fn miss_decrements_attempts() {
    let mut game = Hangman::new("rust", 5);

    assert_eq!(game.guess("x"), GuessResult::Miss);
    assert_eq!(game.attempts_left(), 4);
    assert_eq!(game.masked_word(), "_ _ _ _");
}

#[test]
fn repeated_letter_does_not_cost_attempt() {
    let mut game = Hangman::new("rust", 5);

    assert_eq!(game.guess("r"), GuessResult::Hit);
    assert_eq!(game.attempts_left(), 5);

    assert_eq!(game.guess("r"), GuessResult::AlreadyGuessed);
    assert_eq!(game.attempts_left(), 5);
    assert_eq!(game.masked_word(), "r _ _ _");
}

#[test]
fn empty_and_multi_char_input_are_invalid() {
    let mut game = Hangman::new("rust", 5);

    assert_eq!(game.guess(""), GuessResult::Invalid);
    assert_eq!(game.guess("   "), GuessResult::Invalid);
    assert_eq!(game.guess("ab"), GuessResult::Invalid);
    assert_eq!(game.guess("1"), GuessResult::Invalid);
    assert_eq!(game.attempts_left(), 5);
    assert!(game.guessed_letters().is_empty());
}

#[test]
fn player_wins_when_all_letters_are_guessed() {
    let mut game = Hangman::new("hi", 3);

    assert_eq!(game.guess("h"), GuessResult::Hit);
    assert_eq!(game.guess("i"), GuessResult::Won);
    assert!(game.is_won());
    assert!(!game.is_lost());
    assert_eq!(game.masked_word(), "h i");
    assert_eq!(game.secret_word(), "hi");
}

#[test]
fn player_loses_when_attempts_run_out() {
    let mut game = Hangman::new("ab", 2);

    assert_eq!(game.guess("x"), GuessResult::Miss);
    assert_eq!(game.guess("y"), GuessResult::Lost);
    assert!(game.is_lost());
    assert!(!game.is_won());
    assert_eq!(game.attempts_left(), 0);
    assert_eq!(game.secret_word(), "ab");
}

#[test]
fn guess_is_case_insensitive() {
    let mut game = Hangman::new("Rust", 5);

    assert_eq!(game.guess("R"), GuessResult::Hit);
    assert_eq!(game.masked_word(), "r _ _ _");
    assert_eq!(game.secret_word(), "rust");
}

#[test]
fn empty_word_is_not_an_immediate_win() {
    let game = Hangman::new("", 5);

    assert!(!game.is_won());
    assert!(!game.is_lost());
    assert_eq!(game.masked_word(), "");
}
