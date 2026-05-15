use std::io::{self, Read};

fn main() {
    let mut data = Vec::new();
    io::stdin().read_to_end(&mut data).unwrap();

    let bytes = data.len();
    let mut lines = 0;
    let mut words = 0;
    let mut in_word = false;

    for &byte in &data {
        if byte == b'\n' {
            lines += 1;
        }
        // let is_whitespace = byte == b' ' || byte == b'\t' || byte == b'\n';
        //
        // if !is_whitespace {
        //     if !in_word {
        //         words += 1;
        //         in_word = true;
        //     }
        // } else {
        //     in_word = false;
        // }
        if byte.is_ascii_whitespace() {
            in_word = false;
        } else if !in_word {
            words += 1;
            in_word = true;
        }
    }

    println!("{} {} {}", lines, words, bytes)
}
