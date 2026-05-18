use std::env;
fn standard_sort(args: &[String]) -> Vec<String> {
    let mut sorted_args = args.to_vec();
    sorted_args.sort();
    sorted_args
}

fn insertion_sort(args: &[String]) -> Vec<String> {
    let mut sorted_args = args.to_vec();
    for i in 1..sorted_args.len() {
        let mut j = i;
        while j > 0 && sorted_args[j - 1] > sorted_args[j] {
            sorted_args.swap(j - 1, j);
            j -= 1;
        }
    }
    sorted_args
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    // for arg in standard_sort(&args) {
    //     println!("{}", arg);
    // }
    for arg in insertion_sort(&args) {
        println!("{}", arg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_empty() {
        assert_eq!(insertion_sort(&[]), Vec::<String>::new());
        assert_eq!(standard_sort(&[]), Vec::<String>::new());
    }

    #[test]
    fn case() {
        let args: Vec<String> = vec!["A", "a", "A", "a", "A", "a"]
            .into_iter()
            .map(String::from)
            .collect();
        let expected: Vec<String> = vec!["A", "A", "A", "a", "a", "a"]
            .into_iter()
            .map(String::from)
            .collect();

        assert_eq!(insertion_sort(&args), expected);
        assert_eq!(standard_sort(&args), expected);
    }

    #[test]
    fn default() {
        let args: Vec<String> = vec!["e", "d", "c", "b", "a"]
            .into_iter()
            .map(String::from)
            .collect();
        let expected: Vec<String> = vec!["a", "b", "c", "d", "e"]
            .into_iter()
            .map(String::from)
            .collect();

        assert_eq!(standard_sort(&args), expected);
        assert_eq!(insertion_sort(&args), expected);
    }
}
