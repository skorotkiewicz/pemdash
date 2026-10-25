use std::f64::consts::{E, PI, TAU};

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Number(f64),
    Identifier(String),
    Operator(char),
    LeftParen,
    RightParen,
    Comma,
}

/// Evaluates an expression using operator precedence and finite `f64` arithmetic.
///
/// # Errors
///
/// Returns an error for invalid syntax, unknown names, incorrect argument
/// counts, division by zero, or non-finite numeric literals and results.
///
/// # Examples
///
/// ```
/// assert_eq!(pemdash::calculate("2 + 2 * 2")?, 6.0);
/// # Ok::<(), String>(())
/// ```
pub fn calculate(expression: &str) -> Result<f64, String> {
    let expression = expression.trim();
    let expression = expression
        .strip_suffix('=')
        .unwrap_or(expression)
        .trim_end();
    let tokens = tokenize(expression)?;

    if tokens.is_empty() {
        return Err("Enter an expression".into());
    }

    let mut parser = Parser {
        tokens,
        position: 0,
    };
    let value = parser.parse_expression(0)?;

    if let Some(token) = parser.peek() {
        return Err(format!("Unexpected token: {}", token_name(token)));
    }

    Ok(value)
}

fn tokenize(expression: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = expression.chars().collect();
    let mut tokens = Vec::new();
    let mut position = 0;

    while position < chars.len() {
        let character = chars[position];

        if character.is_whitespace() {
            position += 1;
            continue;
        }

        if character.is_ascii_digit()
            || (character == '.' && chars.get(position + 1).is_some_and(char::is_ascii_digit))
        {
            let start = position;

            while chars.get(position).is_some_and(char::is_ascii_digit) {
                position += 1;
            }
            if chars.get(position) == Some(&'.') {
                position += 1;
                while chars.get(position).is_some_and(char::is_ascii_digit) {
                    position += 1;
                }
            }
            if matches!(chars.get(position), Some('e' | 'E')) {
                position += 1;
                if matches!(chars.get(position), Some('+' | '-')) {
                    position += 1;
                }
                let exponent_start = position;
                while chars.get(position).is_some_and(char::is_ascii_digit) {
                    position += 1;
                }
                if exponent_start == position {
                    return Err("Invalid scientific notation".into());
                }
            }

            let number: String = chars[start..position].iter().collect();
            tokens.push(Token::Number(finite_result(
                number
                    .parse()
                    .map_err(|_| format!("Invalid number: {number}"))?,
            )?));
            continue;
        }

        if character.is_ascii_alphabetic() || character == '_' {
            let start = position;
            position += 1;
            while chars
                .get(position)
                .is_some_and(|c| c.is_ascii_alphanumeric() || *c == '_')
            {
                position += 1;
            }
            tokens.push(Token::Identifier(chars[start..position].iter().collect()));
            continue;
        }

        tokens.push(match character {
            '+' | '-' | '*' | '/' | '^' => Token::Operator(character),
            '(' => Token::LeftParen,
            ')' => Token::RightParen,
            ',' => Token::Comma,
            _ => return Err(format!("Invalid character: '{character}'")),
        });
        position += 1;
    }

    Ok(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.peek()?.clone();
        self.position += 1;
        Some(token)
    }

    fn parse_expression(&mut self, minimum_precedence: u8) -> Result<f64, String> {
        let mut value = self.parse_value()?;

        while let Some(Token::Operator(operator)) = self.peek() {
            let operator = *operator;
            let precedence = precedence(operator);
            if precedence < minimum_precedence {
                break;
            }

            self.position += 1;
            let next_minimum = precedence + u8::from(operator != '^');
            let right = self.parse_expression(next_minimum)?;
            value = apply_operator(operator, value, right)?;
        }

        Ok(value)
    }

    fn parse_value(&mut self) -> Result<f64, String> {
        match self.next() {
            Some(Token::Operator(operator @ ('+' | '-'))) => {
                let value = self.parse_expression(3)?;
                Ok(if operator == '-' { -value } else { value })
            }
            Some(Token::Number(value)) => Ok(value),
            Some(Token::LeftParen) => {
                let value = self.parse_expression(0)?;
                match self.next() {
                    Some(Token::RightParen) => Ok(value),
                    token => Err(format!(
                        "Expected ')', found {}",
                        token.as_ref().map_or("end", token_name)
                    )),
                }
            }
            Some(Token::Identifier(name)) if matches!(self.peek(), Some(Token::LeftParen)) => {
                self.position += 1;
                let arguments = self.parse_arguments()?;
                apply_function(&name, &arguments)
            }
            Some(Token::Identifier(name)) => match name.as_str() {
                "pi" => Ok(PI),
                "e" => Ok(E),
                "tau" => Ok(TAU),
                _ => Err(format!("Unknown identifier: {name}")),
            },
            Some(token) => Err(format!("Unexpected token: {}", token_name(&token))),
            None => Err("Unexpected end of expression".into()),
        }
    }

    fn parse_arguments(&mut self) -> Result<Vec<f64>, String> {
        let mut arguments = Vec::new();

        if matches!(self.peek(), Some(Token::RightParen)) {
            self.position += 1;
            return Ok(arguments);
        }

        loop {
            arguments.push(self.parse_expression(0)?);
            match self.next() {
                Some(Token::Comma) => {}
                Some(Token::RightParen) => return Ok(arguments),
                token => {
                    return Err(format!(
                        "Expected ',' or ')', found {}",
                        token.as_ref().map_or("end", token_name)
                    ));
                }
            }
        }
    }
}

fn precedence(operator: char) -> u8 {
    match operator {
        '+' | '-' => 1,
        '*' | '/' => 2,
        '^' => 4,
        _ => unreachable!(),
    }
}

fn finite_result(value: f64) -> Result<f64, String> {
    if value.is_nan() {
        Err("Result is not a real number".into())
    } else if !value.is_finite() {
        Err("Result is outside the finite range".into())
    } else {
        Ok(value)
    }
}

fn apply_operator(operator: char, left: f64, right: f64) -> Result<f64, String> {
    finite_result(match operator {
        '+' => left + right,
        '-' => left - right,
        '*' => left * right,
        '/' if right == 0.0 => return Err("Division by zero".into()),
        '/' => left / right,
        '^' => left.powf(right),
        _ => unreachable!(),
    })
}

fn apply_function(name: &str, arguments: &[f64]) -> Result<f64, String> {
    let unary = |function: fn(f64) -> f64| match arguments {
        [value] => Ok(function(*value)),
        _ => Err(format!("{name} expects one argument")),
    };
    let binary = |function: fn(f64, f64) -> f64| match arguments {
        [left, right] => Ok(function(*left, *right)),
        _ => Err(format!("{name} expects two arguments")),
    };

    let value = match name {
        "sin" => unary(f64::sin),
        "cos" => unary(f64::cos),
        "tan" => unary(f64::tan),
        "asin" => unary(f64::asin),
        "acos" => unary(f64::acos),
        "atan" => unary(f64::atan),
        "sqrt" => unary(f64::sqrt),
        "abs" => unary(f64::abs),
        "exp" => unary(f64::exp),
        "ln" => unary(f64::ln),
        "log10" => unary(f64::log10),
        "floor" => unary(f64::floor),
        "ceil" => unary(f64::ceil),
        "round" => unary(f64::round),
        "pow" => binary(f64::powf),
        "atan2" => binary(f64::atan2),
        "min" if !arguments.is_empty() => {
            Ok(arguments.iter().copied().fold(f64::INFINITY, f64::min))
        }
        "max" if !arguments.is_empty() => {
            Ok(arguments.iter().copied().fold(f64::NEG_INFINITY, f64::max))
        }
        "min" | "max" => Err(format!("{name} expects at least one argument")),
        _ => Err(format!("Unknown function: {name}")),
    }?;

    finite_result(value)
}

fn token_name(token: &Token) -> &str {
    match token {
        Token::Number(_) => "number",
        Token::Identifier(name) => name,
        Token::Operator('+') => "+",
        Token::Operator('-') => "-",
        Token::Operator('*') => "*",
        Token::Operator('/') => "/",
        Token::Operator('^') => "^",
        Token::Operator(_) => "operator",
        Token::LeftParen => "(",
        Token::RightParen => ")",
        Token::Comma => ",",
    }
}

#[cfg(test)]
mod tests {
    use super::calculate;
    use std::f64::consts::PI;

    fn assert_close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 1e-12, "{actual} != {expected}");
    }

    #[test]
    fn follows_operator_precedence() {
        assert_close(calculate("2 + 2 * 2").unwrap(), 6.0);
        assert_close(calculate("(2 + 2) * 2").unwrap(), 8.0);
        assert_close(calculate("2^3^2").unwrap(), 512.0);
    }

    #[test]
    fn exponent_binds_more_tightly_than_unary_minus() {
        assert_close(calculate("-2^2").unwrap(), -4.0);
        assert_close(calculate("(-2)^2").unwrap(), 4.0);
        assert_close(calculate("2^-3").unwrap(), 0.125);
    }

    #[test]
    fn supports_constants_functions_and_scientific_notation() {
        assert_close(calculate("sin(pi / 2)").unwrap(), 1.0);
        assert_close(calculate("sqrt(9) + 1e2").unwrap(), 103.0);
        assert_close(calculate("max(2, 8, 3) - min(2, 8, 3)").unwrap(), 6.0);
        assert_close(calculate("tau / 2").unwrap(), PI);
    }

    #[test]
    fn accepts_calculator_style_trailing_equals() {
        assert_close(
            calculate("50 / sqrt(3) * 1e5 =").unwrap(),
            2_886_751.345_948_129,
        );
    }

    #[test]
    fn rejects_non_finite_literals_and_results() {
        for expression in [
            "1e999",
            "-1e999",
            "1e308 + 1e308",
            "1e308 * 2",
            "exp(1000)",
            "ln(0)",
        ] {
            assert_eq!(
                calculate(expression).unwrap_err(),
                "Result is outside the finite range",
                "{expression}"
            );
        }
        for expression in ["sqrt(-1)", "ln(-1)", "asin(2)", "(-1)^0.5", "pow(-1, 0.5)"] {
            assert_eq!(
                calculate(expression).unwrap_err(),
                "Result is not a real number",
                "{expression}"
            );
        }
        for expression in ["1 / 0", "0 / 0", "1 / -0"] {
            assert_eq!(
                calculate(expression).unwrap_err(),
                "Division by zero",
                "{expression}"
            );
        }
    }

    #[test]
    fn rejects_invalid_intermediate_results() {
        for expression in [
            "(1 / 0)^0",
            "1 / 1e999",
            "(1e308 * 2)^0",
            "sqrt(-1)^0",
            "min(sqrt(-1), 2)",
            "max(2, sqrt(-1))",
            "pow(exp(1000), 0)",
        ] {
            assert!(calculate(expression).is_err(), "{expression}");
        }
    }

    #[test]
    fn preserves_finite_boundary_values() {
        assert_eq!(calculate("1e308").unwrap(), 1e308);
        assert_eq!(calculate("1e-308").unwrap(), 1e-308);
        assert_eq!(calculate("0 / 2").unwrap(), 0.0);
        assert_eq!(calculate("min(-2, 0, 3)").unwrap(), -2.0);
        assert_eq!(calculate("max(-2, 0, 3)").unwrap(), 3.0);
    }

    #[test]
    fn tokenizes_numbers_and_rejects_invalid_characters() {
        assert_close(calculate(" .5 + 1. + 2E-1 ").unwrap(), 1.7);
        for expression in ["1e", "1e+", "1e-", "2 @ 3", "2pi"] {
            assert!(calculate(expression).is_err(), "{expression}");
        }
    }

    #[test]
    fn rejects_malformed_arguments_and_wrong_arity() {
        for expression in [
            "sqrt(1, 2)",
            "pow(2)",
            "pow(2, 3, 4)",
            "min()",
            "max()",
            "min(,1)",
            "min(1,)",
            "min(1,,2)",
            "min(1 2)",
            "min(1,2",
        ] {
            assert!(calculate(expression).is_err(), "{expression}");
        }
    }

    #[test]
    fn supports_documented_functions() {
        for (expression, expected) in [
            ("asin(0)", 0.0),
            ("acos(1)", 0.0),
            ("atan(0)", 0.0),
            ("atan2(0, 1)", 0.0),
            ("exp(0)", 1.0),
            ("floor(1.9)", 1.0),
            ("ceil(1.1)", 2.0),
            ("round(1.5)", 2.0),
            ("pow(2, 3)", 8.0),
            ("cos(0)", 1.0),
            ("tan(0)", 0.0),
            ("abs(-2)", 2.0),
            ("ln(e)", 1.0),
            ("log10(100)", 2.0),
        ] {
            assert_close(calculate(expression).unwrap(), expected);
        }
    }

    #[test]
    fn rejects_invalid_expressions() {
        assert!(calculate("").is_err());
        assert!(calculate("2 2").is_err());
        assert!(calculate("sqrt()").is_err());
        assert!(calculate("1e+").is_err());
        assert!(calculate("unknown(2)").is_err());
        assert!(calculate("(2 + 2").is_err());
    }
}
