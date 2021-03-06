use thiserror::Error;

pub struct RpnCalculator(bool);

#[derive(Debug, Error)]
pub enum RpnCalculatorError {
    #[error("invalid syntax at {0}")]
    InvalidSyntax(i32),
}

impl RpnCalculator {
    pub fn new(verbose: bool) -> Self {
        Self(verbose)
    }

    pub fn eval(&self, formula: &str) -> Result<i32, RpnCalculatorError> {
        let mut tokens = formula.split_whitespace().rev().collect::<Vec<_>>();
        self.eval_inner(&mut tokens)
    }

    fn eval_inner(&self, tokens: &mut Vec<&str>) -> Result<i32, RpnCalculatorError> {
        let mut stack = Vec::new();
        let mut pos = 0;

        while let Some(token) = tokens.pop() {
            pos += 1;

            if let Ok(x) = token.parse::<i32>() {
                stack.push(x);
            } else {
                let y = stack.pop().ok_or(RpnCalculatorError::InvalidSyntax(pos))?;
                let x = stack.pop().ok_or(RpnCalculatorError::InvalidSyntax(pos))?;
                let res = match token {
                    "+" => x + y,
                    "-" => x - y,
                    "*" => x * y,
                    "/" => x / y,
                    "%" => x % y,
                    _ => return Err(RpnCalculatorError::InvalidSyntax(pos)),
                };
                stack.push(res);
            }

            if self.0 {
                println!("{:?} {:?}", tokens, stack);
            }
        }

        if stack.len() == 1 {
            Ok(stack[0])
        } else {
            Err(RpnCalculatorError::InvalidSyntax(-1))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case("5", 5)]
    #[case("50", 50)]
    #[case("-50", -50)]
    fn test_one_operand(#[case] input: &str, #[case] expected: i32) {
        let calc = RpnCalculator::new(false);
        assert_eq!(calc.eval(input).unwrap(), expected);
    }

    #[rstest]
    #[case("2 3 +", 5)]
    #[case("2 3 *", 6)]
    #[case("2 3 -", -1)]
    #[case("2 3 /", 0)]
    #[case("2 3 %", 2)]
    fn test_two_operand(#[case] input: &str, #[case] expected: i32) {
        let calc = RpnCalculator::new(false);
        assert_eq!(calc.eval(input).unwrap(), expected);
    }

    #[rstest]
    #[case("")]
    #[case("1 1 1 +")]
    #[case("+ 1 1")]
    #[case("1 1 ^")]
    fn test_error(#[case] input: &str) {
        let calc = RpnCalculator::new(false);
        assert!(calc.eval(input).is_err());
    }
}
