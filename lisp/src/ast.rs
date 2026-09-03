use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;


#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    Bool(bool),
    Symbol(String),
    List(Vec<Expr>),
}

pub type Env = Rc<RefCell<Environment>>;

pub type BuiltinFn = fn(&[Value]) -> Result<Value, String>;

#[derive(Clone)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Function(Rc<Closure>),
    Builtin(BuiltinFn),
    Nil,
}

#[derive(Clone)]
pub struct Closure {
    pub params: Vec<String>,
    pub body: Vec<Expr>,
    pub env: Env,
}

pub struct Environment {
    pub vars: HashMap<String, Value>,
    pub parent: Option<Env>,
}

pub fn format_value(value: &Value) -> String {
    match value {
        Value::Number(n) => {
            if n.fract() == 0.0 && n.is_finite() {
                format!("{n:.0}")
            } else {
                format!("{n}")
            }
        }
        Value::Bool(b) => b.to_string(),
        Value::Function(_) => "<function>".to_string(),
        Value::Builtin(_) => "<builtin>".to_string(),
        Value::Nil => "nil".to_string(),
    }
}

pub fn expect_number(value: &Value) -> Result<f64, String> {
    match value {
        Value::Number(n) => Ok(*n),
        _ => Err("expected number".to_string()),
    }
}

pub fn is_truthy(value: &Value) -> bool {
    !matches!(value, Value::Bool(false) | Value::Nil)
}
