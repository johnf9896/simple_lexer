use crate::parser::{Location, NodeType, ParseNode};
use std::collections::HashMap;
use std::f64::EPSILON;

#[derive(Debug, PartialEq, Fail)]
pub enum EvalError {
    #[fail(display = "Unimplemented: {}", _0)]
    Unimplemented(String),
    #[fail(display = "Symbol '{}' not found at {:?}", _0, _1)]
    SymbolNotFound(String, Location),
}

type EvalResult = Result<f64, EvalError>;
type SymbolTable = HashMap<String, f64>;

pub struct EvalContext {
    syms: SymbolTable,
}

impl EvalContext {
    pub fn new() -> Self {
        EvalContext {
            syms: HashMap::new(),
        }
    }

    pub fn populated() -> Self {
        let mut ctx = Self::new();
        ctx.populate_symbol_table();
        ctx
    }

    pub fn eval(&mut self, node: &ParseNode) -> EvalResult {
        use NodeType::*;
        match &node.ntype {
            Root(nodes) => {
                let mut res = None;
                for node in nodes {
                    res = Some(self.eval(node)?);
                }

                Ok(res.unwrap())
            }
            Number(num) => Ok(*num),
            Sum(left, right) => self.perform_arithmetic_op(left, right, |l, r| l + r),
            Substraction(left, right) => self.perform_arithmetic_op(left, right, |l, r| l - r),
            Multiplication(left, right) => self.perform_arithmetic_op(left, right, |l, r| l * r),
            Division(left, right) => self.perform_arithmetic_op(left, right, |l, r| l / r),
            GreaterThan(left, right) => self.perform_comparison_op(left, right, |l, r| l > r),
            GreaterThanOrEqual(left, right) => {
                self.perform_comparison_op(left, right, |l, r| l >= r)
            }
            LessThan(left, right) => self.perform_comparison_op(left, right, |l, r| l < r),
            LessThanOrEqual(left, right) => self.perform_comparison_op(left, right, |l, r| l <= r),
            Equal(left, right) => {
                self.perform_comparison_op(left, right, |l, r| (l - r).abs() < EPSILON)
            }
            Assignment(identifier, right) => {
                let val = self.eval(right)?;
                self.syms.insert(identifier.clone(), val);
                Ok(val)
            }
            Identifier(identifier) => self
                .syms
                .get(identifier)
                .copied()
                .ok_or_else(|| EvalError::SymbolNotFound(identifier.clone(), node.location)),
        }
    }

    pub fn eval_and_print(root: &ParseNode) -> Result<(), EvalError> {
        let nodes = match &root.ntype {
            NodeType::Root(nodes) => nodes,
            _ => panic!("Expected Root node, got {:?}", root),
        };

        let mut ctx = Self::populated();

        println!();

        for node in nodes {
            let res = ctx.eval(node)?;
            println!("{}", res);
        }

        Ok(())
    }

    fn perform_arithmetic_op<F>(
        &mut self,
        left_child: &ParseNode,
        right_child: &ParseNode,
        op: F,
    ) -> EvalResult
    where
        F: FnOnce(f64, f64) -> f64,
    {
        let left_res = self.eval(&left_child)?;
        let right_res = self.eval(&right_child)?;
        Ok(op(left_res, right_res))
    }

    fn perform_comparison_op<F>(
        &mut self,
        left_child: &ParseNode,
        right_child: &ParseNode,
        op: F,
    ) -> EvalResult
    where
        F: FnOnce(f64, f64) -> bool,
    {
        let left_res = self.eval(&left_child)?;
        let right_res = self.eval(&right_child)?;
        let res = op(left_res, right_res);
        Ok(if res { 1.0 } else { 0.0 })
    }

    fn populate_symbol_table(&mut self) {
        self.syms.insert(String::from("PI"), std::f64::consts::PI);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::{NodeType, Parser};

    fn eval(input: &str) -> EvalResult {
        let tokens = Lexer::get_tokens(input).unwrap();
        let mut parser = Parser::new(&tokens);
        let root = parser.parse().unwrap();
        let node = if let NodeType::Root(_) = &root.ntype {
            &root
        } else {
            panic!("Parse result should always be a Root node");
        };

        let mut ctx = EvalContext::populated();
        ctx.eval(node)
    }

    fn assert_res(lhs: EvalResult, rhs: EvalResult) {
        match (lhs, rhs) {
            (Ok(a), Ok(b)) => assert!((a - b).abs() <= EPSILON),
            (lhs, rhs) => assert_eq!(lhs, rhs),
        }
    }

    #[test]
    fn test_eval_number() {
        assert_res(eval("3.2"), Ok(3.2));
    }

    #[test]
    fn test_eval_sum() {
        assert_res(eval("3.2 + 2.0"), Ok(5.2));
    }

    #[test]
    fn test_eval_substraction() {
        assert_res(eval("3.2 - 2.0"), Ok(1.2));
    }

    #[test]
    fn test_eval_multiplication() {
        assert_res(eval("3.2 * 2.0"), Ok(6.4));
    }

    #[test]
    fn test_eval_division() {
        assert_res(eval("3.2 / 2.0"), Ok(1.6));
    }

    #[test]
    fn test_eval_greater_than() {
        assert_res(eval("3.2 > 2.0"), Ok(1.0));
    }

    #[test]
    fn test_eval_greater_than_false() {
        assert_res(eval("3.2 > 5.0"), Ok(0.0));
    }

    #[test]
    fn test_eval_greater_or_equal_than() {
        assert_res(eval("3.2 >= 2.0"), Ok(1.0));
    }

    #[test]
    fn test_eval_greater_or_equal_than_false() {
        assert_res(eval("3.2 >= 5.0"), Ok(0.0));
    }

    #[test]
    fn test_eval_less_than() {
        assert_res(eval("3.2 < 4.0"), Ok(1.0));
    }

    #[test]
    fn test_eval_less_than_false() {
        assert_res(eval("3.2 < 1.0"), Ok(0.0));
    }

    #[test]
    fn test_eval_less_or_equal_than() {
        assert_res(eval("3.2 <= 4.0"), Ok(1.0));
    }

    #[test]
    fn test_eval_less_or_equal_than_false() {
        assert_res(eval("3.2 <= 1.0"), Ok(0.0));
    }

    #[test]
    fn test_eval_equal() {
        assert_res(eval("3.2 == 3.2"), Ok(1.0));
    }

    #[test]
    fn test_eval_equal_false() {
        assert_res(eval("3.2 == 5.0"), Ok(0.0));
    }

    #[test]
    fn test_eval_pi() {
        assert_res(eval("PI >= 3.14159265358979312"), Ok(1.0));
    }

    #[test]
    fn test_assinment() {
        assert_res(eval("hello = 5.0"), Ok(5.0));
    }

    #[test]
    fn test_assinment2() {
        assert_res(eval("hello = 5.0\n2.3 + hello"), Ok(7.3));
    }

    #[test]
    fn test_nested() {
        assert_res(eval("(6 * 5) / 4 + 2"), Ok(9.5));
        assert_res(eval("(2 * 5) / 3 - 2"), Ok(1.33333333333333333));
        assert_res(eval("(6 * 5) / 4 + (8 * PI)"), Ok(32.632741228718345));
        assert_res(eval("(6 * 5) / 4 + (8 * PI) > 32"), Ok(1.0));
        assert_res(eval("(((((3)))))"), Ok(3.0));
    }
}
