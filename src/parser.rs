use crate::repr::*;
use regex::Regex;
use std::sync::LazyLock;

#[derive(Clone)]
struct Parser<'s>(&'s str);

impl<'s> Parser<'s> {
    fn consume(&self, p: char) -> Option<Self> {
        self.0.strip_prefix(p).map(Parser)
    }

    fn next(&self) -> Option<(char, Self)> {
        self.0.chars().next().map(|c| (c, Parser(&self.0[1..])))
    }

    fn spaces(&self) -> Self {
        match self.consume(' ') {
            Some(p) => p.spaces(),
            None => self.clone(),
        }
    }

    fn float(&self) -> Option<(f64, Self)> {
        static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^-?\d+(\.\d+)?").unwrap());
        let s = &RE.captures(self.0)?.get(0)?.as_str();
        s.parse::<f64>()
            .ok()
            .map(|f| (f, Parser(&self.0[s.len()..])))
    }

    fn term(&self) -> Option<(Operand, Self)> {
        if let Some((c, p)) = self.next() {
            match c {
                '(' => {
                    let (e, p) = p.expr().ok()?;
                    let p = p.consume(')')?;
                    Some((e, p))
                }
                _ => self.float().map(|(f, p)| (f.into(), p)),
            }
        } else {
            None
        }
    }

    fn expr(&self) -> Result<(Operand, Self), String> {
        let p = self;
        let p = p.spaces();
        let (a, p) = p.term().ok_or("invalid term")?;
        let p = p.spaces();
        if let Ok((op, b, p)) = Ok::<_, String>(()).and_then(|_| {
            let (op, p) = p.next().ok_or("invalid operator")?;
            let (b, p) = p.expr()?;
            let op = match op {
                '+' => Operator::Add,
                '-' => Operator::Sub,
                '*' => Operator::Mul,
                '/' => Operator::Div,
                _ => return Err(format!("unsupported operator: {}", op))?,
            };
            Ok((op, b, p))
        }) {
            let node = match op {
                Operator::Add | Operator::Sub | Operator::Mul | Operator::Div => {
                    Node::new(op, [a, b])
                }
            };
            Ok((node.into(), p))
        } else {
            Ok((a, p))
        }
    }
}

pub fn parse_line(line: &str) -> Result<Operand, String> {
    let (res, p) = Parser(line).expr()?;
    let p = p.spaces();
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
            assert_eq!(Parser(input).spaces().0, output, "input was {input:?}",);
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
            let res = Parser(input).float().map(|(a, p)| (a, p.0));
            if let Some((output, rest)) = expected {
                assert_eq!(res, Some((output, rest)), "parsing failed for {input:?}");
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
                    parse_line(input).unwrap(),
                    Node::new(Operator::Add, [a.into(), b.into()]).into(),
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
                    Ok(Node::new(Operator::Add, [a.into(), b.into()]).into()),
                    "failed to parse {input:?}"
                );
            }
        }

        #[test]
        fn add_3() {
            for (input, (a, b, c)) in [("1+2+3", (1.0, 2.0, 3.0)), ("1 + 2 + 3", (1.0, 2.0, 3.0))] {
                assert_eq!(
                    parse_line(input),
                    Ok(Node::new(
                        Operator::Add,
                        [
                            a.into(),
                            Node::new(Operator::Add, [b.into(), c.into()]).into()
                        ]
                    )
                    .into()),
                    "failed to parse {input:?}"
                );
            }
        }

        #[test]
        fn add_3_paren() {
            assert_eq!(
                parse_line("(1+2)+3"),
                Ok(Node::new(
                    Operator::Add,
                    [
                        Node::new(Operator::Add, [1.0.into(), 2.0.into()]).into(),
                        3.0.into(),
                    ]
                )
                .into()),
                "failed to parse {:?}",
                "(1+2)+3"
            );
            assert_eq!(
                parse_line("1+(2+3)"),
                Ok(Node::new(
                    Operator::Add,
                    [
                        1.0.into(),
                        Node::new(Operator::Add, [2.0.into(), 3.0.into()]).into()
                    ]
                )
                .into()),
                "failed to parse {:?}",
                "1+(2+3)"
            );
        }
    }
}
