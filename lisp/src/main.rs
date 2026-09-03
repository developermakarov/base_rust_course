mod ast;
mod builtins;
mod env;
mod eval;
mod parser;

use std::io::{self, Write};
use std::rc::Rc;

use ast::format_value;
use env::default_env;
use eval::run_line;

fn main() {
    let env = default_env();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("> ");
        if stdout.flush().is_err() {
            break;
        }

        let mut line = String::new();
        match stdin.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => break,
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "quit" || trimmed == "exit" {
            break;
        }

        match run_line(trimmed, Rc::clone(&env)) {
            Ok(value) => println!("{}", format_value(&value)),
            Err(err) => println!("Error: {err}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Environment, Value, is_truthy};
    use crate::parser::{parse, tokenize};

    fn eval_str(line: &str) -> Result<Value, String> {
        run_line(line, default_env())
    }

    fn eval_lines(lines: &[&str]) -> Result<Value, String> {
        let env = default_env();
        let mut last = Value::Nil;
        for line in lines {
            last = run_line(line, Rc::clone(&env))?;
        }
        Ok(last)
    }

    fn expect_number(line: &str, expected: f64) {
        match eval_str(line).expect("eval failed") {
            Value::Number(n) => assert!((n - expected).abs() < f64::EPSILON),
            other => panic!("expected number, got {}", format_value(&other)),
        }
    }

    fn expect_bool(line: &str, expected: bool) {
        match eval_str(line).expect("eval failed") {
            Value::Bool(b) => assert_eq!(b, expected),
            other => panic!("expected bool, got {}", format_value(&other)),
        }
    }

    #[test]
    fn tokenize_lambda_expression() {
        let tokens = tokenize("(lambda (x) (+ x 1))");
        assert_eq!(
            tokens,
            vec!["(", "lambda", "(", "x", ")", "(", "+", "x", "1", ")", ")"]
        );
    }

    #[test]
    fn parse_simple_forms() {
        assert_eq!(
            parse(&tokenize("42")).unwrap(),
            crate::ast::Expr::Number(42.0)
        );
        assert_eq!(
            parse(&tokenize("true")).unwrap(),
            crate::ast::Expr::Bool(true)
        );
        assert_eq!(
            parse(&tokenize("x")).unwrap(),
            crate::ast::Expr::Symbol("x".to_string())
        );
    }

    #[test]
    fn arithmetic_add() {
        expect_number("(+ 1 2)", 3.0);
    }

    #[test]
    fn arithmetic_nested() {
        expect_number("(* (+ 2 3) 4)", 20.0);
    }

    #[test]
    fn comparisons() {
        expect_bool("(= 2 2)", true);
        expect_bool("(< 1 2)", true);
        expect_bool("(> 5 3)", true);
        expect_bool("(< 10 3)", false);
    }

    #[test]
    fn define_and_lookup() {
        match eval_lines(&["(define x 10)", "(+ x 5)"]).unwrap() {
            Value::Number(n) => assert!((n - 15.0).abs() < f64::EPSILON),
            other => panic!("unexpected {}", format_value(&other)),
        }
    }

    #[test]
    fn lambda_call() {
        match eval_lines(&["(define add (lambda (a b) (+ a b)))", "(add 2 3)"]).unwrap() {
            Value::Number(n) => assert!((n - 5.0).abs() < f64::EPSILON),
            other => panic!("unexpected {}", format_value(&other)),
        }
    }

    #[test]
    fn immediate_lambda_call() {
        expect_number("((lambda (x) (+ x 1)) 10)", 11.0);
    }

    #[test]
    fn functions_as_values() {
        match eval_lines(&[
            "(define apply-twice (lambda (f x) (f (f x))))",
            "(define inc (lambda (x) (+ x 1)))",
            "(apply-twice inc 10)",
        ])
        .unwrap()
        {
            Value::Number(n) => assert!((n - 12.0).abs() < f64::EPSILON),
            other => panic!("unexpected {}", format_value(&other)),
        }
    }

    #[test]
    fn if_true_and_false() {
        expect_number("(if true 1 2)", 1.0);
        expect_number("(if false 1 2)", 2.0);
        expect_number("(if (> 5 3) 10 20)", 10.0);
    }

    #[test]
    fn let_creates_local_scope() {
        match eval_lines(&["(define x 10)", "(let ((x 100) (y 5)) (+ x y))"]).unwrap() {
            Value::Number(n) => assert!((n - 105.0).abs() < f64::EPSILON),
            other => panic!("unexpected {}", format_value(&other)),
        }

        match eval_lines(&["(define x 10)", "(let ((x 100) (y 5)) (+ x y))", "x"]).unwrap() {
            Value::Number(n) => assert!((n - 10.0).abs() < f64::EPSILON),
            other => panic!("unexpected {}", format_value(&other)),
        }
    }

    #[test]
    fn lexical_scoping() {
        match eval_lines(&[
            "(define x 10)",
            "(define f (lambda (y) (+ x y)))",
            "(let ((x 100)) (f 1))",
        ])
        .unwrap()
        {
            Value::Number(n) => assert!((n - 11.0).abs() < f64::EPSILON),
            other => panic!("unexpected {}", format_value(&other)),
        }
    }

    #[test]
    fn closures_make_adder() {
        let env = default_env();
        run_line(
            "(define make-adder (lambda (x) (lambda (y) (+ x y))))",
            Rc::clone(&env),
        )
        .unwrap();
        run_line("(define add10 (make-adder 10))", Rc::clone(&env)).unwrap();
        run_line("(define add20 (make-adder 20))", Rc::clone(&env)).unwrap();

        match run_line("(add10 5)", Rc::clone(&env)).unwrap() {
            Value::Number(n) => assert!((n - 15.0).abs() < f64::EPSILON),
            other => panic!("unexpected {}", format_value(&other)),
        }
        match run_line("(add20 5)", Rc::clone(&env)).unwrap() {
            Value::Number(n) => assert!((n - 25.0).abs() < f64::EPSILON),
            other => panic!("unexpected {}", format_value(&other)),
        }
    }

    #[test]
    fn factorial_recursion() {
        match eval_lines(&[
            "(define fact (lambda (n) (if (= n 0) 1 (* n (fact (- n 1))))))",
            "(fact 5)",
        ])
        .unwrap()
        {
            Value::Number(n) => assert!((n - 120.0).abs() < f64::EPSILON),
            other => panic!("unexpected {}", format_value(&other)),
        }
    }

    #[test]
    fn set_and_counter_closure() {
        let env = default_env();
        run_line("(define x 10)", Rc::clone(&env)).unwrap();
        match run_line("(set! x 20)", Rc::clone(&env)).unwrap() {
            Value::Number(n) => assert!((n - 20.0).abs() < f64::EPSILON),
            other => panic!("unexpected {}", format_value(&other)),
        }
        match run_line("x", Rc::clone(&env)).unwrap() {
            Value::Number(n) => assert!((n - 20.0).abs() < f64::EPSILON),
            other => panic!("unexpected {}", format_value(&other)),
        }

        run_line(
            "(define make-counter (lambda () (let ((count 0)) (lambda () (set! count (+ count 1)) count))))",
            Rc::clone(&env),
        )
        .unwrap();
        run_line("(define c1 (make-counter))", Rc::clone(&env)).unwrap();
        run_line("(define c2 (make-counter))", Rc::clone(&env)).unwrap();

        for (line, expected) in [("(c1)", 1.0), ("(c1)", 2.0), ("(c2)", 1.0), ("(c1)", 3.0)] {
            match run_line(line, Rc::clone(&env)).unwrap() {
                Value::Number(n) => assert!((n - expected).abs() < f64::EPSILON),
                other => panic!("unexpected {}", format_value(&other)),
            }
        }
    }

    #[test]
    fn division_by_zero_error() {
        match eval_str("(/ 10 0)") {
            Err(err) => assert!(err.contains("division by zero")),
            Ok(value) => panic!("expected error, got {}", format_value(&value)),
        }
    }

    #[test]
    fn arity_and_unknown_variable_errors() {
        match eval_str("(- 1 2 3)") {
            Err(err) => assert!(err.contains("exactly 2 arguments")),
            Ok(value) => panic!("expected error, got {}", format_value(&value)),
        }

        match eval_str("missing") {
            Err(err) => assert!(err.contains("unknown variable: missing")),
            Ok(value) => panic!("expected error, got {}", format_value(&value)),
        }

        match run_line("(set! missing 1)", default_env()) {
            Err(err) => assert!(err.contains("unknown variable: missing")),
            Ok(value) => panic!("expected error, got {}", format_value(&value)),
        }
    }

    #[test]
    fn format_value_and_truthiness() {
        assert_eq!(format_value(&Value::Number(10.0)), "10");
        assert_eq!(format_value(&Value::Number(2.5)), "2.5");
        assert_eq!(format_value(&Value::Bool(true)), "true");
        assert_eq!(format_value(&Value::Nil), "nil");
        assert!(!is_truthy(&Value::Bool(false)));
        assert!(!is_truthy(&Value::Nil));
        assert!(is_truthy(&Value::Number(0.0)));
    }

    #[test]
    fn environment_lookup_and_assign_through_parent() {
        let global = Environment::new();
        Environment::define(&global, "x".to_string(), Value::Number(10.0));
        let child = Environment::extend(Rc::clone(&global));

        match Environment::lookup(&child, "x") {
            Some(Value::Number(n)) => assert!((n - 10.0).abs() < f64::EPSILON),
            _ => panic!("lookup failed"),
        }

        Environment::assign(&child, "x", Value::Number(20.0)).unwrap();
        match Environment::lookup(&global, "x") {
            Some(Value::Number(n)) => assert!((n - 20.0).abs() < f64::EPSILON),
            _ => panic!("assign did not update parent"),
        }
    }

    #[test]
    fn empty_lambda_body_returns_nil() {
        match eval_str("((lambda ()))").unwrap() {
            Value::Nil => {}
            other => panic!("expected nil, got {}", format_value(&other)),
        }
    }
}
