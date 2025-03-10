
#[derive(Debug, PartialEq, Clone)]
pub enum Operand {
    Float(f64),
    Node(Node),
}

impl Operand {
    pub fn evaluate(&self) -> Result<f64, String> {
        match self {
            Operand::Float(f) => Ok(*f),
            Operand::Node(n) => n.evaluate(),
        }
    }
}

impl From<f64> for Operand {
    fn from(val: f64) -> Self {
        Operand::Float(val)
    }
}

impl From<Node> for Operand {
    fn from(val: Node) -> Self {
        Operand::Node(val)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Node {
    op: Operator,
    params: Vec<Operand>,
}

impl Node {
    pub fn new(op: Operator, params: impl IntoIterator<Item = Operand>) -> Self {
        Self {
            op,
            params: params.into_iter().collect(),
        }
    }
}

impl Node {
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
        assert_f64_near!(Node::new(Operator::Add, [2.3.into(), 4.1.into()]).evaluate().unwrap(), 6.4);
    }

    #[test]
    fn sub_basic() {
        assert_f64_near!(Node::new(Operator::Sub, [2.3.into(), 4.1.into()]).evaluate().unwrap(), -1.8);
    }

    #[test]
    fn mul_basic() {
        assert_f64_near!(Node::new(Operator::Mul, [2.3.into(), 4.1.into()]).evaluate().unwrap(), 9.43);
    }
    
    #[test]
    fn div_basic() {
        assert_f64_near!(Node::new(Operator::Div, [2.3.into(), 4.1.into()]).evaluate().unwrap(), 0.560975609756098);
    }
    
    #[test]
    fn div_zero() {
        // TODO: should there be an error instead?
        assert_f64_near!(Node::new(Operator::Div, [2.3.into(), 0.0.into()]).evaluate().unwrap(), f64::INFINITY);
        assert_f64_near!(Node::new(Operator::Div, [2.3.into(), (-0.0).into()]).evaluate().unwrap(), -f64::INFINITY);
    }
}
