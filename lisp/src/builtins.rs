use crate::ast::{Value, expect_number};

pub fn builtin_add(args: &[Value]) -> Result<Value, String> {
    let mut sum = 0.0;
    for arg in args {
        sum += expect_number(arg)?;
    }
    Ok(Value::Number(sum))
}

pub fn builtin_sub(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("'-' expects exactly 2 arguments".to_string());
    }
    Ok(Value::Number(
        expect_number(&args[0])? - expect_number(&args[1])?,
    ))
}

pub fn builtin_mul(args: &[Value]) -> Result<Value, String> {
    let mut product = 1.0;
    for arg in args {
        product *= expect_number(arg)?;
    }
    Ok(Value::Number(product))
}

pub fn builtin_div(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("'/' expects exactly 2 arguments".to_string());
    }
    let dividend = expect_number(&args[0])?;
    let divisor = expect_number(&args[1])?;
    if divisor == 0.0 {
        return Err("division by zero".to_string());
    }
    Ok(Value::Number(dividend / divisor))
}

pub fn builtin_eq(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("'=' expects exactly 2 arguments".to_string());
    }
    Ok(Value::Bool(
        expect_number(&args[0])? == expect_number(&args[1])?,
    ))
}

pub fn builtin_lt(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("'<' expects exactly 2 arguments".to_string());
    }
    Ok(Value::Bool(
        expect_number(&args[0])? < expect_number(&args[1])?,
    ))
}

pub fn builtin_gt(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("'>' expects exactly 2 arguments".to_string());
    }
    Ok(Value::Bool(
        expect_number(&args[0])? > expect_number(&args[1])?,
    ))
}
