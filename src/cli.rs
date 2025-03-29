use std::io::{BufRead, Write};

use colored::Colorize;

use crate::parser;

pub fn run_cli(stdin: &mut impl BufRead, stdout: &mut impl Write) -> Result<(), std::io::Error> {
    let mut line = String::new();

    loop {
        write!(stdout, "> ")?;
        stdout.flush().unwrap();

        line.clear();
        let bytes_read = stdin.read_line(&mut line).unwrap();
        if bytes_read == 0 {
            // EOF
            return Ok(());
        }
        let line = line.strip_suffix('\n').unwrap_or_else(|| &line);

        match parser::parse_line(line) {
            Ok(v) => match v.evaluate() {
                Ok(res) => writeln!(stdout, "{}", res.to_string().green())?,
                Err(err) => writeln!(stdout, "{}", err.red())?,
            },
            Err(err) => writeln!(stdout, "{}", err.red())?,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use super::*;

    mod ansi {
        pub const RESET: &str = "\u{1b}[0m";
        pub const FG_RED: &str = "\u{1b}[31m";
        pub const FG_GREEN: &str = "\u{1b}[32m";
    }

    #[test]
    fn cli_success() {
        let input = "1 + 2";
        let expected_output = [
            // initial prompt
            "> ",
            // answer (with color)
            &[ansi::FG_GREEN, "3", ansi::RESET, "\n"].concat(),
            // next prompt
            "> ",
        ];

        let mut output = Vec::new();
        run_cli(&mut BufReader::new(input.as_bytes()), &mut output).unwrap();

        assert_eq!(String::from_utf8(output), Ok(expected_output.concat()));
    }

    #[test]
    fn cli_syntax_error() {
        let input = "1 + *";
        let expected_output = [
            // initial prompt
            "> ",
            // error message (with color)
            &[
                ansi::FG_RED,
                r#"invalid term: "*""#,
                ansi::RESET,
                "\n",
            ]
            .concat(),
            // next prompt
            "> ",
        ];

        let mut output = Vec::new();
        run_cli(&mut BufReader::new(input.as_bytes()), &mut output).unwrap();

        assert_eq!(String::from_utf8(output), Ok(expected_output.concat()));
    }

    #[test]
    fn cli_math_error() {
        let input = "1 / 0";
        let expected_output = [
            // initial prompt
            "> ",
            // error message (with color)
            &[ansi::FG_GREEN, "inf", ansi::RESET, "\n"].concat(),
            // next prompt
            "> ",
        ];

        let mut output = Vec::new();
        run_cli(&mut BufReader::new(input.as_bytes()), &mut output).unwrap();

        assert_eq!(String::from_utf8(output), Ok(expected_output.concat()));
    }

    #[test]
    fn cli_multiple_prompts() {
        let input = "1 + 2\n3 * 4";
        let expected_output = [
            // initial prompt
            "> ",
            // answer (with color)
            &[ansi::FG_GREEN, "3", ansi::RESET, "\n"].concat(),
            // next prompt
            "> ",
            // answer (with color)
            &[ansi::FG_GREEN, "12", ansi::RESET, "\n"].concat(),
            // next prompt
            "> ",
        ];

        let mut output = Vec::new();
        run_cli(&mut BufReader::new(input.as_bytes()), &mut output).unwrap();

        assert_eq!(String::from_utf8(output), Ok(expected_output.concat()));
    }
}
