
#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Float(f64),
    Op(Operation),
}

impl Expr {
    pub fn evaluate(&self) -> Result<f64, String> {
        match self {
            Expr::Float(f) => Ok(*f),
            Expr::Op(n) => n.evaluate(),
        }
    }
}

impl From<f64> for Expr {
    fn from(val: f64) -> Self {
        Expr::Float(val)
    }
}

impl From<Operation> for Expr {
    fn from(val: Operation) -> Self {
        Expr::Op(val)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
}

impl Operator {
    /// lower value means operator is applied sooner
    pub fn precedence(self) -> u8 {
        match self {
            Operator::Add => 2,
            Operator::Sub => 2,
            Operator::Mul => 1,
            Operator::Div => 1,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Operation {
    op: Operator,
    params: Vec<Expr>,
}

impl Operation {
    pub fn new(op: Operator, params: impl IntoIterator<Item = Expr>) -> Self {
        Self {
            op,
            params: params.into_iter().collect(),
        }
    }
}

impl Operation {
    pub fn evaluate(&self) -> Result<f64, String> {
        let res = match self.op {
            Operator::Add => self
                .evaluate_params()?
                .into_iter()
                .reduce(|a, b| a + b)
                .unwrap(),
            Operator::Sub => self
                .evaluate_params()?
                .into_iter()
                .reduce(|a, b| a - b)
                .unwrap(),
            Operator::Mul => self
                .evaluate_params()?
                .into_iter()
                .reduce(|a, b| a * b)
                .unwrap(),
            Operator::Div => self
                .evaluate_params()?
                .into_iter()
                .reduce(|a, b| a / b)
                .unwrap(),
        };
        Ok(res)
    }

    fn evaluate_params(&self) -> Result<Vec<f64>, String> {
        let mut res = Vec::with_capacity(self.params.len());
        for p in &self.params {
            res.push(p.evaluate()?);
        }
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use assert_float_eq::assert_f64_near;

    use super::*;

    #[test]
    fn add_basic() {
        assert_f64_near!(Operation::new(Operator::Add, [2.3.into(), 4.1.into()]).evaluate().unwrap(), 6.4);
    }

    #[test]
    fn sub_basic() {
        assert_f64_near!(Operation::new(Operator::Sub, [2.3.into(), 4.1.into()]).evaluate().unwrap(), -1.8);
    }

    #[test]
    fn mul_basic() {
        assert_f64_near!(Operation::new(Operator::Mul, [2.3.into(), 4.1.into()]).evaluate().unwrap(), 9.43);
    }
    
    #[test]
    fn div_basic() {
        assert_f64_near!(Operation::new(Operator::Div, [2.3.into(), 4.1.into()]).evaluate().unwrap(), 0.560975609756098);
    }
    
    #[test]
    fn div_zero() {
        // TODO: should there be an error instead?
        assert_f64_near!(Operation::new(Operator::Div, [2.3.into(), 0.0.into()]).evaluate().unwrap(), f64::INFINITY);
        assert_f64_near!(Operation::new(Operator::Div, [2.3.into(), (-0.0).into()]).evaluate().unwrap(), -f64::INFINITY);
    }
}
