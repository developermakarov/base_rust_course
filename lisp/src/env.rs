use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub use crate::ast::Environment;
use crate::ast::{Env, Value};
use crate::builtins::{
    builtin_add, builtin_div, builtin_eq, builtin_gt, builtin_lt, builtin_mul, builtin_sub,
};

impl Environment {
    pub fn new() -> Env {
        Rc::new(RefCell::new(Environment {
            vars: HashMap::new(),
            parent: None,
        }))
    }

    pub fn extend(parent: Env) -> Env {
        Rc::new(RefCell::new(Environment {
            vars: HashMap::new(),
            parent: Some(parent),
        }))
    }

    pub fn lookup(env: &Env, name: &str) -> Option<Value> {
        let borrowed = env.borrow();
        if let Some(value) = borrowed.vars.get(name) {
            return Some(value.clone());
        }
        borrowed
            .parent
            .as_ref()
            .and_then(|parent| Environment::lookup(parent, name))
    }

    pub fn define(env: &Env, name: String, value: Value) {
        env.borrow_mut().vars.insert(name, value);
    }

    pub fn assign(env: &Env, name: &str, value: Value) -> Result<(), String> {
        let mut current = Rc::clone(env);
        loop {
            {
                let mut borrowed = current.borrow_mut();
                if borrowed.vars.contains_key(name) {
                    borrowed.vars.insert(name.to_string(), value);
                    return Ok(());
                }
                if let Some(parent) = borrowed.parent.as_ref() {
                    let next = Rc::clone(parent);
                    drop(borrowed);
                    current = next;
                    continue;
                }
            }
            return Err(format!("unknown variable: {name}"));
        }
    }
}

pub fn default_env() -> Env {
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
