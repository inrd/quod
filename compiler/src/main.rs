#![forbid(unsafe_code)]

use std::process::ExitCode;

const USAGE: &str = "\
usage: quodc <command> [options] <file>...

commands:
  fmt [--check | --write] <file>...   print canonical format (default: to stdout)
                                      --check: list non-canonical files, exit 1 if any
                                      --write: rewrite files in place
  parse <file>                        print the AST as an S-expression
  check <file>                        type-check; print the unverified-checks report
  lower <file>                        lower to Core IR (thin-slice subset); print its text
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some((cmd, rest)) = args.split_first() else {
        eprint!("{USAGE}");
        return ExitCode::from(2);
    };
    match cmd.as_str() {
        "fmt" => fmt(rest),
        "parse" => parse(rest),
        "check" => check(rest),
        "lower" => lower(rest),
        "-h" | "--help" | "help" => {
            print!("{USAGE}");
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("quodc: unknown command `{other}`\n");
            eprint!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn read(path: &str) -> Result<String, ExitCode> {
    std::fs::read_to_string(path).map_err(|e| {
        eprintln!("quodc: cannot read {path}: {e}");
        ExitCode::from(2)
    })
}

fn fmt(args: &[String]) -> ExitCode {
    let (mut check, mut write) = (false, false);
    let mut files = Vec::new();
    for a in args {
        match a.as_str() {
            "--check" => check = true,
            "--write" => write = true,
            f if f.starts_with('-') => {
                eprintln!("quodc fmt: unknown option `{f}`");
                return ExitCode::from(2);
            }
            f => files.push(f),
        }
    }
    if files.is_empty() || (check && write) {
        eprint!("{USAGE}");
        return ExitCode::from(2);
    }
    let mut status = ExitCode::SUCCESS;
    for path in files {
        let src = match read(path) {
            Ok(s) => s,
            Err(code) => return code,
        };
        let formatted = match quodc::format_source(&src) {
            Ok(f) => f,
            Err(d) => {
                eprintln!("{}", d.render(path, &src));
                status = ExitCode::from(1);
                continue;
            }
        };
        if check {
            if formatted != src {
                println!("{path}");
                status = ExitCode::from(1);
            }
        } else if write {
            if formatted != src {
                if let Err(e) = std::fs::write(path, &formatted) {
                    eprintln!("quodc: cannot write {path}: {e}");
                    return ExitCode::from(2);
                }
            }
        } else {
            print!("{formatted}");
        }
    }
    status
}

fn parse(args: &[String]) -> ExitCode {
    let [path] = args else {
        eprint!("{USAGE}");
        return ExitCode::from(2);
    };
    let src = match read(path) {
        Ok(s) => s,
        Err(code) => return code,
    };
    match quodc::parser::parse(&src) {
        Ok(file) => {
            print!("{}", quodc::dump::dump(&file));
            ExitCode::SUCCESS
        }
        Err(d) => {
            eprintln!("{}", d.render(path, &src));
            ExitCode::from(1)
        }
    }
}

fn check(args: &[String]) -> ExitCode {
    let [path] = args else {
        eprint!("{USAGE}");
        return ExitCode::from(2);
    };
    let src = match read(path) {
        Ok(s) => s,
        Err(code) => return code,
    };
    let result = quodc::parser::parse(&src).and_then(|f| quodc::check::check(&f));
    match result {
        Ok(checked) => {
            print!(
                "{}",
                quodc::check::render_report(path, &src, &checked.traps)
            );
            ExitCode::SUCCESS
        }
        Err(d) => {
            eprintln!("{}", d.render(path, &src));
            ExitCode::from(1)
        }
    }
}

fn lower(args: &[String]) -> ExitCode {
    let [path] = args else {
        eprint!("{USAGE}");
        return ExitCode::from(2);
    };
    let src = match read(path) {
        Ok(s) => s,
        Err(code) => return code,
    };
    match quodc::lower_source(&src) {
        Ok(text) => {
            print!("{text}");
            ExitCode::SUCCESS
        }
        Err(Err(d)) => {
            eprintln!("{}", d.render(path, &src));
            ExitCode::from(1)
        }
        Err(Ok(internal)) => {
            eprintln!("quodc: internal error: {internal}");
            ExitCode::from(3)
        }
    }
}
