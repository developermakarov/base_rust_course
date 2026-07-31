use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{self, Write};
use std::rc::Rc;

type Env = Rc<RefCell<Environment>>;
type BuiltinFn = fn(&[Value]) -> Result<Value, String>;

#[derive(Debug, Clone, PartialEq)]
enum Expr {
    Number(f64),
    Bool(bool),
    Symbol(String),
    List(Vec<Expr>),
}

#[derive(Clone)]
enum Value {
    Number(f64),
    Bool(bool),
    Function(Rc<Closure>),
    Builtin(BuiltinFn),
    Nil,
}

#[derive(Clone)]
struct Closure {
    params: Vec<String>,
    body: Vec<Expr>,
    env: Env,
}

struct Environment {
    vars: HashMap<String, Value>,
    parent: Option<Env>,
}

impl Environment {
    fn new() -> Env {
        Rc::new(RefCell::new(Environment {
            vars: HashMap::new(),
            parent: None,
        }))
    }

    fn extend(parent: Env) -> Env {
        Rc::new(RefCell::new(Environment {
            vars: HashMap::new(),
            parent: Some(parent),
        }))
    }

    fn lookup(env: &Env, name: &str) -> Option<Value> {
        let current = env.borrow();
        if let Some(value) = current.vars.get(name) {
            return Some(value.clone());
        }
        match &current.parent {
            Some(parent) => Environment::lookup(parent, name),
            None => None,
        }
    }

    fn define(env: &Env, name: String, value: Value) {
        env.borrow_mut().vars.insert(name, value);
    }

    fn assign(env: &Env, name: &str, value: Value) -> Result<(), String> {
        let mut current = env.borrow_mut();
        if current.vars.contains_key(name) {
            current.vars.insert(name.to_string(), value);
            return Ok(());
        }
        match &current.parent {
            Some(parent) => {
                let parent = Rc::clone(parent);
                drop(current);
                Environment::assign(&parent, name, value)
            }
            None => Err(format!("unknown variable: {name}")),
        }
    }
}

fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for ch in input.chars() {
        match ch {
            '(' | ')' => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
                tokens.push(ch.to_string());
            }
            c if c.is_whitespace() => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

fn parse(tokens: &[String]) -> Result<Expr, String> {
    if tokens.is_empty() {
        return Err("unexpected end of input".to_string());
    }
    let (expr, next) = parse_expr(tokens, 0)?;
    if next != tokens.len() {
        return Err("unexpected tokens after expression".to_string());
    }
    Ok(expr)
}

fn parse_expr(tokens: &[String], pos: usize) -> Result<(Expr, usize), String> {
    let token = tokens
        .get(pos)
        .ok_or_else(|| "unexpected end of input".to_string())?;

    match token.as_str() {
        "(" => parse_list(tokens, pos + 1),
        ")" => Err("unexpected ')'".to_string()),
        "true" => Ok((Expr::Bool(true), pos + 1)),
        "false" => Ok((Expr::Bool(false), pos + 1)),
        other => {
            if let Ok(number) = other.parse::<f64>() {
                Ok((Expr::Number(number), pos + 1))
            } else {
                Ok((Expr::Symbol(other.to_string()), pos + 1))
            }
        }
    }
}

fn parse_list(tokens: &[String], start: usize) -> Result<(Expr, usize), String> {
    let mut items = Vec::new();
    let mut pos = start;

    while pos < tokens.len() {
        if tokens[pos] == ")" {
            return Ok((Expr::List(items), pos + 1));
        }
        let (expr, next) = parse_expr(tokens, pos)?;
        items.push(expr);
        pos = next;
    }

    Err("missing closing ')'".to_string())
}

fn default_env() -> Env {
    let env = Environment::new();
    Environment::define(&env, "+".to_string(), Value::Builtin(builtin_add));
    Environment::define(&env, "-".to_string(), Value::Builtin(builtin_sub));
    Environment::define(&env, "*".to_string(), Value::Builtin(builtin_mul));
    Environment::define(&env, "/".to_string(), Value::Builtin(builtin_div));
    Environment::define(&env, "=".to_string(), Value::Builtin(builtin_eq));
    Environment::define(&env, "<".to_string(), Value::Builtin(builtin_lt));
    Environment::define(&env, ">".to_string(), Value::Builtin(builtin_gt));
    env
}

fn expect_number(value: &Value) -> Result<f64, String> {
    match value {
        Value::Number(n) => Ok(*n),
        _ => Err("expected number".to_string()),
    }
}

fn builtin_add(args: &[Value]) -> Result<Value, String> {
    let mut sum = 0.0;
    for arg in args {
        sum += expect_number(arg)?;
    }
    Ok(Value::Number(sum))
}

fn builtin_sub(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("'-' expects exactly 2 arguments".to_string());
    }
    Ok(Value::Number(
        expect_number(&args[0])? - expect_number(&args[1])?,
    ))
}

fn builtin_mul(args: &[Value]) -> Result<Value, String> {
    let mut product = 1.0;
    for arg in args {
        product *= expect_number(arg)?;
    }
    Ok(Value::Number(product))
}

fn builtin_div(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("'/' expects exactly 2 arguments".to_string());
    }
    let divisor = expect_number(&args[1])?;
    if divisor == 0.0 {
        return Err("division by zero".to_string());
    }
    Ok(Value::Number(expect_number(&args[0])? / divisor))
}

fn builtin_eq(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("'=' expects exactly 2 arguments".to_string());
    }
    Ok(Value::Bool(
        expect_number(&args[0])? == expect_number(&args[1])?,
    ))
}

fn builtin_lt(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("'<' expects exactly 2 arguments".to_string());
    }
    Ok(Value::Bool(
        expect_number(&args[0])? < expect_number(&args[1])?,
    ))
}

fn builtin_gt(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("'>' expects exactly 2 arguments".to_string());
    }
    Ok(Value::Bool(
        expect_number(&args[0])? > expect_number(&args[1])?,
    ))
}

fn is_truthy(value: &Value) -> bool {
    !matches!(value, Value::Bool(false) | Value::Nil)
}

fn format_value(value: &Value) -> String {
    match value {
        Value::Number(n) => {
            if n.fract() == 0.0 && n.is_finite() {
                format!("{}", *n as i64)
            } else {
                n.to_string()
            }
        }
        Value::Bool(b) => b.to_string(),
        Value::Function(_) => "<function>".to_string(),
        Value::Builtin(_) => "<builtin>".to_string(),
        Value::Nil => "nil".to_string(),
    }
}

fn eval(expr: &Expr, env: Env) -> Result<Value, String> {
    match expr {
        Expr::Number(n) => Ok(Value::Number(*n)),
        Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::Symbol(name) => {
            Environment::lookup(&env, name).ok_or_else(|| format!("unknown variable: {name}"))
        }
        Expr::List(items) => eval_list(items, env),
    }
}

fn eval_sequence(body: &[Expr], env: Env) -> Result<Value, String> {
    if body.is_empty() {
        return Ok(Value::Nil);
    }

    let mut result = Value::Nil;
    for expr in body {
        result = eval(expr, Rc::clone(&env))?;
    }
    Ok(result)
}

fn eval_list(items: &[Expr], env: Env) -> Result<Value, String> {
    if items.is_empty() {
        return Ok(Value::Nil);
    }

    match &items[0] {
        Expr::Symbol(name) => match name.as_str() {
            "define" => eval_define(items, env),
            "lambda" => eval_lambda(items, env),
            "if" => eval_if(items, env),
            "let" => eval_let(items, env),
            "set!" => eval_set(items, env),
            _ => eval_call(items, env),
        },
        _ => eval_call(items, env),
    }
}

fn eval_define(items: &[Expr], env: Env) -> Result<Value, String> {
    if items.len() != 3 {
        return Err("'define' expects exactly 2 arguments".to_string());
    }

    let name = match &items[1] {
        Expr::Symbol(name) => name.clone(),
        _ => return Err("'define' expects a symbol name".to_string()),
    };

    let value = eval(&items[2], Rc::clone(&env))?;
    Environment::define(&env, name, value.clone());
    Ok(value)
}

fn eval_set(items: &[Expr], env: Env) -> Result<Value, String> {
    if items.len() != 3 {
        return Err("'set!' expects exactly 2 arguments".to_string());
    }

    let name = match &items[1] {
        Expr::Symbol(name) => name.clone(),
        _ => return Err("'set!' expects a symbol name".to_string()),
    };

    let value = eval(&items[2], Rc::clone(&env))?;
    Environment::assign(&env, &name, value.clone())?;
    Ok(value)
}

fn eval_lambda(items: &[Expr], env: Env) -> Result<Value, String> {
    if items.len() < 3 {
        return Err("'lambda' expects parameters and body".to_string());
    }

    let params = match &items[1] {
        Expr::List(params) => {
            let mut names = Vec::with_capacity(params.len());
            for param in params {
                match param {
                    Expr::Symbol(name) => names.push(name.clone()),
                    _ => return Err("'lambda' parameters must be symbols".to_string()),
                }
            }
            names
        }
        _ => return Err("'lambda' expects a parameter list".to_string()),
    };

    let body = items[2..].to_vec();
    Ok(Value::Function(Rc::new(Closure { params, body, env })))
}

fn eval_if(items: &[Expr], env: Env) -> Result<Value, String> {
    if items.len() != 4 {
        return Err("'if' expects condition, then, and else".to_string());
    }

    let condition = eval(&items[1], Rc::clone(&env))?;
    if is_truthy(&condition) {
        eval(&items[2], env)
    } else {
        eval(&items[3], env)
    }
}

fn eval_let(items: &[Expr], env: Env) -> Result<Value, String> {
    if items.len() < 3 {
        return Err("'let' expects bindings and body".to_string());
    }

    let bindings = match &items[1] {
        Expr::List(bindings) => bindings,
        _ => return Err("'let' expects a list of bindings".to_string()),
    };

    let local_env = Environment::extend(Rc::clone(&env));
    for binding in bindings {
        match binding {
            Expr::List(pair) if pair.len() == 2 => {
                let name = match &pair[0] {
                    Expr::Symbol(name) => name.clone(),
                    _ => return Err("'let' binding name must be a symbol".to_string()),
                };
                let value = eval(&pair[1], Rc::clone(&env))?;
                Environment::define(&local_env, name, value);
            }
            _ => return Err("'let' bindings must be (name expr) pairs".to_string()),
        }
    }

    eval_sequence(&items[2..], local_env)
}

fn eval_call(items: &[Expr], env: Env) -> Result<Value, String> {
    let func = eval(&items[0], Rc::clone(&env))?;
    let mut args = Vec::with_capacity(items.len().saturating_sub(1));
    for arg in &items[1..] {
        args.push(eval(arg, Rc::clone(&env))?);
    }
    apply(func, args)
}

fn apply(func: Value, args: Vec<Value>) -> Result<Value, String> {
    match func {
        Value::Builtin(builtin) => builtin(&args),
        Value::Function(closure) => {
            if args.len() != closure.params.len() {
                return Err(format!(
                    "function expects {} arguments, got {}",
                    closure.params.len(),
                    args.len()
                ));
            }

            let call_env = Environment::extend(Rc::clone(&closure.env));
            for (param, arg) in closure.params.iter().zip(args.into_iter()) {
                Environment::define(&call_env, param.clone(), arg);
            }
            eval_sequence(&closure.body, call_env)
        }
        _ => Err("attempt to call a non-function value".to_string()),
    }
}

fn run_line(line: &str, env: Env) -> Result<Value, String> {
    let tokens = tokenize(line);
    if tokens.is_empty() {
        return Ok(Value::Nil);
    }
    let expr = parse(&tokens)?;
    eval(&expr, env)
}

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

    fn eval_str(env: &Env, line: &str) -> Value {
        run_line(line, Rc::clone(env)).expect(line)
    }

    fn eval_number(env: &Env, line: &str) -> f64 {
        match eval_str(env, line) {
            Value::Number(n) => n,
            other => panic!(
                "expected number from `{line}`, got {}",
                format_value(&other)
            ),
        }
    }

    fn eval_bool(env: &Env, line: &str) -> bool {
        match eval_str(env, line) {
            Value::Bool(b) => b,
            other => panic!("expected bool from `{line}`, got {}", format_value(&other)),
        }
    }

    #[test]
    fn tokenize_splits_parentheses() {
        assert_eq!(
            tokenize("(lambda (x) (+ x 1))"),
            vec!["(", "lambda", "(", "x", ")", "(", "+", "x", "1", ")", ")"]
        );
    }

    #[test]
    fn parse_number_bool_symbol() {
        assert_eq!(parse(&tokenize("42")).unwrap(), Expr::Number(42.0));
        assert_eq!(parse(&tokenize("true")).unwrap(), Expr::Bool(true));
        assert_eq!(
            parse(&tokenize("x")).unwrap(),
            Expr::Symbol("x".to_string())
        );
    }

    #[test]
    fn parse_lambda_expression() {
        let expr = parse(&tokenize("(lambda (x) (+ x 1))")).unwrap();
        assert_eq!(
            expr,
            Expr::List(vec![
                Expr::Symbol("lambda".to_string()),
                Expr::List(vec![Expr::Symbol("x".to_string())]),
                Expr::List(vec![
                    Expr::Symbol("+".to_string()),
                    Expr::Symbol("x".to_string()),
                    Expr::Number(1.0),
                ]),
            ])
        );
    }

    #[test]
    fn environment_lookup_and_assign_through_parent() {
        let global = Environment::new();
        Environment::define(&global, "x".to_string(), Value::Number(10.0));
        let child = Environment::extend(Rc::clone(&global));

        match Environment::lookup(&child, "x") {
            Some(Value::Number(n)) => assert_eq!(n, 10.0),
            _ => panic!("expected number"),
        }

        Environment::assign(&child, "x", Value::Number(20.0)).unwrap();
        match Environment::lookup(&global, "x") {
            Some(Value::Number(n)) => assert_eq!(n, 20.0),
            _ => panic!("expected updated number"),
        }

        let err = Environment::assign(&child, "missing", Value::Number(1.0)).unwrap_err();
        assert_eq!(err, "unknown variable: missing");
    }

    #[test]
    fn arithmetic_and_nested_calls() {
        let env = default_env();
        assert_eq!(eval_number(&env, "(+ 1 2)"), 3.0);
        assert_eq!(eval_number(&env, "(* (+ 2 3) 4)"), 20.0);
        assert_eq!(eval_number(&env, "(- 10 3)"), 7.0);
        assert_eq!(eval_number(&env, "(/ 20 4)"), 5.0);
    }

    #[test]
    fn comparisons() {
        let env = default_env();
        assert!(eval_bool(&env, "(= 2 2)"));
        assert!(eval_bool(&env, "(< 1 2)"));
        assert!(eval_bool(&env, "(> 5 3)"));
        assert!(!eval_bool(&env, "(< 10 3)"));
    }

    #[test]
    fn define_and_lookup() {
        let env = default_env();
        eval_str(&env, "(define x 10)");
        assert_eq!(eval_number(&env, "(+ x 5)"), 15.0);
    }

    #[test]
    fn lambda_call_and_immediate_application() {
        let env = default_env();
        eval_str(&env, "(define add (lambda (a b) (+ a b)))");
        assert_eq!(eval_number(&env, "(add 2 3)"), 5.0);
        assert_eq!(eval_number(&env, "((lambda (x) (+ x 1)) 10)"), 11.0);
    }

    #[test]
    fn functions_as_values() {
        let env = default_env();
        eval_str(&env, "(define apply-twice (lambda (f x) (f (f x))))");
        eval_str(&env, "(define inc (lambda (x) (+ x 1)))");
        assert_eq!(eval_number(&env, "(apply-twice inc 10)"), 12.0);
    }

    #[test]
    fn if_form() {
        let env = default_env();
        assert_eq!(eval_number(&env, "(if true 1 2)"), 1.0);
        assert_eq!(eval_number(&env, "(if false 1 2)"), 2.0);
        assert_eq!(eval_number(&env, "(if (> 5 3) 10 20)"), 10.0);
    }

    #[test]
    fn let_creates_local_scope() {
        let env = default_env();
        eval_str(&env, "(define x 10)");
        assert_eq!(eval_number(&env, "(let ((x 100) (y 5)) (+ x y))"), 105.0);
        assert_eq!(eval_number(&env, "x"), 10.0);
    }

    #[test]
    fn lexical_scoping() {
        let env = default_env();
        eval_str(&env, "(define x 10)");
        eval_str(&env, "(define f (lambda (y) (+ x y)))");
        assert_eq!(eval_number(&env, "(let ((x 100)) (f 1))"), 11.0);
    }

    #[test]
    fn closures_make_adder() {
        let env = default_env();
        eval_str(
            &env,
            "(define make-adder (lambda (x) (lambda (y) (+ x y))))",
        );
        eval_str(&env, "(define add10 (make-adder 10))");
        eval_str(&env, "(define add20 (make-adder 20))");
        assert_eq!(eval_number(&env, "(add10 5)"), 15.0);
        assert_eq!(eval_number(&env, "(add20 5)"), 25.0);
    }

    #[test]
    fn recursive_factorial() {
        let env = default_env();
        eval_str(
            &env,
            "(define fact (lambda (n) (if (= n 0) 1 (* n (fact (- n 1))))))",
        );
        assert_eq!(eval_number(&env, "(fact 5)"), 120.0);
    }

    #[test]
    fn set_and_counter_closure() {
        let env = default_env();
        eval_str(&env, "(define x 10)");
        eval_str(&env, "(set! x 20)");
        assert_eq!(eval_number(&env, "x"), 20.0);

        eval_str(
            &env,
            "(define make-counter (lambda () (let ((count 0)) (lambda () (set! count (+ count 1)) count))))",
        );
        eval_str(&env, "(define c1 (make-counter))");
        eval_str(&env, "(define c2 (make-counter))");
        assert_eq!(eval_number(&env, "(c1)"), 1.0);
        assert_eq!(eval_number(&env, "(c1)"), 2.0);
        assert_eq!(eval_number(&env, "(c2)"), 1.0);
        assert_eq!(eval_number(&env, "(c1)"), 3.0);
    }

    #[test]
    fn builtin_errors() {
        let env = default_env();
        match run_line("(/ 10 0)", Rc::clone(&env)) {
            Err(err) => assert_eq!(err, "division by zero"),
            Ok(_) => panic!("expected division by zero"),
        }
        match run_line("(- 1 2 3)", Rc::clone(&env)) {
            Err(err) => assert_eq!(err, "'-' expects exactly 2 arguments"),
            Ok(_) => panic!("expected arity error"),
        }
    }

    #[test]
    fn format_value_renders_common_cases() {
        assert_eq!(format_value(&Value::Number(10.0)), "10");
        assert_eq!(format_value(&Value::Number(2.5)), "2.5");
        assert_eq!(format_value(&Value::Bool(true)), "true");
        assert_eq!(format_value(&Value::Nil), "nil");
        assert_eq!(format_value(&Value::Builtin(builtin_add)), "<builtin>");
    }

    #[test]
    fn is_truthy_rules() {
        assert!(!is_truthy(&Value::Bool(false)));
        assert!(!is_truthy(&Value::Nil));
        assert!(is_truthy(&Value::Bool(true)));
        assert!(is_truthy(&Value::Number(0.0)));
    }
}
