use crate::repr::*;
use regex::Regex;
use std::sync::LazyLock;

#[derive(Clone)]
struct Parser<'s>(&'s str);

impl<'s> Parser<'s> {
    fn attempt<T>(&mut self, f: impl FnOnce(&mut Parser<'s>) -> Option<T>) -> Option<T> {
        let mut p = self.clone();
        let res = f(&mut p);
        if res.is_some() {
            *self = p;
        }
        res
    }

    fn consume(&mut self, p: char) -> Option<()> {
        if let Some(rest) = self.0.strip_prefix(p) {
            self.0 = rest;
            Some(())
        } else {
            None
        }
    }

    fn next(&mut self) -> Option<char> {
        let c = self.0.chars().next();
        if c.is_some() {
            self.0 = &self.0[1..];
        }
        c
    }

    fn spaces(&mut self) {
        while self.consume(' ').is_some() {}
    }

    fn float(&mut self) -> Option<f64> {
        static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^-?\d+(\.\d+)?").unwrap());
        let s = &RE.captures(self.0)?.get(0)?.as_str();
        let f = s.parse::<f64>().ok();
        if f.is_some() {
            self.0 = &self.0[s.len()..];
        }
        f
    }

    fn term(&mut self) -> Option<Expr> {
        match self.clone().next()? {
            '(' => {
                self.consume('(')?;
                let e = self.expr(100).ok()?;
                self.consume(')')?;
                Some(e)
            }
            _ => {
                self.float().map(|f| f.into())
            },
        }
    }

    fn expr(&mut self, max_precedence: u8) -> Result<Expr, String> {
        self.spaces();
        let mut a = self.term().ok_or_else(|| format!("invalid term: {:?}", self.0))?;

        loop {
            self.spaces();
            if self.0.is_empty() {
                break;
            }
            let Some(op) = self.attempt(|p| {
                let op = match p.next()? {
                    '+' => Operator::Add,
                    '-' => Operator::Sub,
                    '*' => Operator::Mul,
                    '/' => Operator::Div,
                    _ => return None,
                };
                if op.precedence() >= max_precedence {
                    return None;
                }
                Some(op)
            }) else {
                break;
            };
            self.spaces();
            let b = self.expr(op.precedence())?;

            a = Operation::new(op, [a, b]).into();
        }

        Ok(a)
    }
}

pub fn parse_line(line: &str) -> Result<Expr, String> {
    let mut p = Parser(line);
    let res = p.expr(100)?;
    p.spaces();
    if !p.0.is_empty() {
        Err(format!(
            "could not parse the end of the imput, namely: {:?}",
            p.0
        ))
    } else {
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_spaces() {
        for (input, output) in [
            ("abc", "abc"),
            (" abc", "abc"),
            ("  abc", "abc"), //
        ] {
            let mut p = Parser(input);
            p.spaces();
            assert_eq!(p.0, output, "input was {input:?}",);
        }
    }

    #[test]
    fn parse_float() {
        for (input, expected) in [
            ("1", Some((1.0, ""))),
            ("1.2", Some((1.2, ""))),
            ("-1.2", Some((-1.2, ""))),
            ("-1.2 ", Some((-1.2, " "))),
            ("-1.2+3.4", Some((-1.2, "+3.4"))),
            ("-1.5.abc", Some((-1.5, ".abc"))),
            ("-1.abc", Some((-1.0, ".abc"))),
            ("+1.2", None),
        ] {
            let mut p = Parser(input);

            let res = p.float();
            if let Some((output, rest)) = expected {
                assert_eq!((res, p.0), (Some(output), rest), "parsing failed for {input:?}");
            } else {
                assert_eq!(res, None, "parsing did not fail for {input:?}");
            }
        }
    }

    mod expr {
        use super::*;

        #[test]
        fn add_2() {
            for (input, (a, b)) in [("1+2", (1.0, 2.0)), ("1 + 2", (1.0, 2.0))] {
                assert_eq!(
                    parse_line(input),
                    Ok(Operation::new(Operator::Add, [a.into(), b.into()]).into()),
                    "failed to parse {input:?}"
                );
            }
        }

        #[test]
        fn add_2_paren() {
            for (input, (a, b)) in [
                ("(1)+2", (1.0, 2.0)),
                ("1+(2)", (1.0, 2.0)),
                ("(1)+(2)", (1.0, 2.0)),
                ("(-1.5)+(-2.5)", (-1.5, -2.5)),
            ] {
                assert_eq!(
                    parse_line(input),
                    Ok(Operation::new(Operator::Add, [a.into(), b.into()]).into()),
                    "failed to parse {input:?}"
                );
            }
        }

        #[test]
        fn add_3() {
            for (input, (a, b, c)) in [("1+2+3", (1.0, 2.0, 3.0)), ("1 + 2 + 3", (1.0, 2.0, 3.0))] {
                assert_eq!(
                    parse_line(input),
                    Ok(Operation::new(
                        Operator::Add,
                        [
                            Operation::new(Operator::Add, [a.into(), b.into()]).into(),
                            c.into(),
                        ]
                    )
                    .into()),
                    "failed to parse {input:?}"
                );
            }
        }

        #[test]
        fn add_4() {
            for (input, (a, b, c, d)) in [
                ("1+2+3+4", (1.0, 2.0, 3.0, 4.0)),
                ("1 + 2 + 3 + 4", (1.0, 2.0, 3.0, 4.0)),
            ] {
                // a + b + c + d = (((a + b) + c) + d)
                let _a = a.into();
                let _b = b.into();
                let _c = c.into();
                let _d = d.into();
                let _ab = Operation::new(Operator::Add, [_a, _b]).into();
                let _abc = Operation::new(Operator::Add, [_ab, _c]).into();
                let _abcd = Operation::new(Operator::Add, [_abc, _d]).into();
                assert_eq!(parse_line(input), Ok(_abcd), "failed to parse {input:?}");
            }
        }

        #[test]
        fn add_3_paren() {
            assert_eq!(
                parse_line("(1+2)+3"),
                Ok(Operation::new(
                    Operator::Add,
                    [
                        Operation::new(Operator::Add, [1.0.into(), 2.0.into()]).into(),
                        3.0.into(),
                    ]
                )
                .into()),
                "failed to parse {:?}",
                "(1+2)+3"
            );
            assert_eq!(
                parse_line("1+(2+3)"),
                Ok(Operation::new(
                    Operator::Add,
                    [
                        1.0.into(),
                        Operation::new(Operator::Add, [2.0.into(), 3.0.into()]).into()
                    ]
                )
                .into()),
                "failed to parse {:?}",
                "1+(2+3)"
            );
        }

        #[test]
        fn add_mul_order() {
            assert_eq!(
                parse_line("1*2+3"),
                Ok(Operation::new(
                    Operator::Add,
                    [
                        Operation::new(Operator::Mul, [1.0.into(), 2.0.into()]).into(),
                        3.0.into(),
                    ]
                )
                .into()),
                "failed to parse {:?}",
                "1*2+3"
            );
            assert_eq!(
                parse_line("1+2*3"),
                Ok(Operation::new(
                    Operator::Add,
                    [
                        1.0.into(),
                        Operation::new(Operator::Mul, [2.0.into(), 3.0.into()]).into(),
                    ]
                )
                .into()),
                "failed to parse {:?}",
                "1+2*3"
            );
        }
    }
}
