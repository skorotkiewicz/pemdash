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
            tokens.push(Token::Number(
                number
                    .parse()
                    .map_err(|_| format!("Invalid number: {number}"))?,
            ));
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
            value = apply_operator(operator, value, right);
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

fn apply_operator(operator: char, left: f64, right: f64) -> f64 {
    match operator {
        '+' => left + right,
        '-' => left - right,
        '*' => left * right,
        '/' => left / right,
        '^' => left.powf(right),
        _ => unreachable!(),
    }
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

    match name {
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
    }
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
    fn rejects_invalid_expressions() {
        assert!(calculate("").is_err());
        assert!(calculate("2 2").is_err());
        assert!(calculate("sqrt()").is_err());
        assert!(calculate("1e+").is_err());
        assert!(calculate("unknown(2)").is_err());
        assert!(calculate("(2 + 2").is_err());
    }
}
