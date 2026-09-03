use std::rc::Rc;

use crate::ast::{Closure, Env, Environment, Expr, Value};
use crate::parser::{parse, tokenize};

pub fn eval(expr: &Expr, env: Env) -> Result<Value, String> {
    match expr {
        Expr::Number(n) => Ok(Value::Number(*n)),
        Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::Symbol(name) => {
            Environment::lookup(&env, name).ok_or_else(|| format!("unknown variable: {name}"))
        }
        Expr::List(items) => eval_list(items, env),
    }
}

pub fn eval_sequence(body: &[Expr], env: Env) -> Result<Value, String> {
    if body.is_empty() {
        return Ok(Value::Nil);
    }

    let mut result = Value::Nil;
    for expr in body {
        result = eval(expr, Rc::clone(&env))?;
    }
    Ok(result)
}

pub fn eval_list(items: &[Expr], env: Env) -> Result<Value, String> {
    if items.is_empty() {
        return Err("cannot evaluate empty list".to_string());
    }

    if let Expr::Symbol(name) = &items[0] {
        match name.as_str() {
            "define" => return eval_define(items, env),
            "lambda" => return eval_lambda(items, env),
            "if" => return eval_if(items, env),
            "let" => return eval_let(items, env),
            "set!" => return eval_set(items, env),
            _ => {}
        }
    }

    eval_call(items, env)
}

pub fn eval_define(items: &[Expr], env: Env) -> Result<Value, String> {
    if items.len() != 3 {
        return Err("'define' expects exactly 2 arguments".to_string());
    }

    let name = match &items[1] {
        Expr::Symbol(name) => name.clone(),
        _ => return Err("'define' expects a symbol name".to_string()),
    };

    let value = eval(&items[2], Rc::clone(&env))?;
    // Создаём/перезаписываем binding только в текущем frame.
    Environment::define(&env, name, value.clone());
    Ok(value)
}

pub fn eval_set(items: &[Expr], env: Env) -> Result<Value, String> {
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

pub fn eval_lambda(items: &[Expr], env: Env) -> Result<Value, String> {
    if items.len() < 2 {
        return Err("'lambda' expects a parameter list".to_string());
    }

    let params = match &items[1] {
        Expr::List(param_exprs) => {
            let mut names = Vec::with_capacity(param_exprs.len());
            for param in param_exprs {
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

pub fn eval_if(items: &[Expr], env: Env) -> Result<Value, String> {
    if items.len() != 4 {
        return Err("'if' expects condition, then and else expressions".to_string());
    }

    let condition = eval(&items[1], Rc::clone(&env))?;
    if crate::ast::is_truthy(&condition) {
        eval(&items[2], env)
    } else {
        eval(&items[3], env)
    }
}

pub fn eval_let(items: &[Expr], env: Env) -> Result<Value, String> {
    if items.len() < 3 {
        return Err("'let' expects bindings and at least one body expression".to_string());
    }

    let bindings = match &items[1] {
        Expr::List(bindings) => bindings,
        _ => return Err("'let' expects a list of bindings".to_string()),
    };

    let local_env = Environment::extend(Rc::clone(&env));
    for binding in bindings {
        let Expr::List(pair) = binding else {
            return Err("'let' binding must be a (name expr) pair".to_string());
        };
        if pair.len() != 2 {
            return Err("'let' binding must be a (name expr) pair".to_string());
        }

        let name = match &pair[0] {
            Expr::Symbol(name) => name.clone(),
            _ => return Err("'let' binding name must be a symbol".to_string()),
        };

        let value = eval(&pair[1], Rc::clone(&env))?;
        Environment::define(&local_env, name, value);
    }

    eval_sequence(&items[2..], local_env)
}

pub fn eval_call(items: &[Expr], env: Env) -> Result<Value, String> {
    let func = eval(&items[0], Rc::clone(&env))?;
    let mut args = Vec::with_capacity(items.len() - 1);
    for arg_expr in &items[1..] {
        args.push(eval(arg_expr, Rc::clone(&env))?);
    }
    apply(func, args)
}

pub fn apply(func: Value, args: Vec<Value>) -> Result<Value, String> {
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
            for (param, arg) in closure.params.iter().zip(args) {
                Environment::define(&call_env, param.clone(), arg);
            }
            eval_sequence(&closure.body, call_env)
        }
        _ => Err("attempt to call a non-function value".to_string()),
    }
}

pub fn run_line(line: &str, env: Env) -> Result<Value, String> {
    let tokens = tokenize(line);
    let expr = parse(&tokens)?;
    eval(&expr, env)
}
