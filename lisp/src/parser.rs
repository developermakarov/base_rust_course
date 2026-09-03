use crate::ast::Expr;

pub fn tokenize(input: &str) -> Vec<String> {
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

pub fn parse(tokens: &[String]) -> Result<Expr, String> {
    if tokens.is_empty() {
        return Err("unexpected end of input".to_string());
    }
    let (expr, next) = parse_expr(tokens, 0)?;
    if next != tokens.len() {
        return Err("unexpected tokens after expression".to_string());
    }
    Ok(expr)
}

pub fn parse_expr(tokens: &[String], pos: usize) -> Result<(Expr, usize), String> {
    let token = tokens
        .get(pos)
        .ok_or_else(|| "unexpected end of input".to_string())?;

    match token.as_str() {
        "(" => parse_list(tokens, pos + 1),
        ")" => Err("unexpected ')'".to_string()),
        "true" => Ok((Expr::Bool(true), pos + 1)),
        "false" => Ok((Expr::Bool(false), pos + 1)),
        _ => {
            if let Ok(number) = token.parse::<f64>() {
                Ok((Expr::Number(number), pos + 1))
            } else {
                Ok((Expr::Symbol(token.clone()), pos + 1))
            }
        }
    }
}

pub fn parse_list(tokens: &[String], start: usize) -> Result<(Expr, usize), String> {
    let mut items = Vec::new();
    let mut pos = start;

    while let Some(token) = tokens.get(pos) {
        if token == ")" {
            return Ok((Expr::List(items), pos + 1));
        }
        let (expr, next) = parse_expr(tokens, pos)?;
        items.push(expr);
        pos = next;
    }

    Err("missing ')'".to_string())
}
