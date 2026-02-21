use lazy_static::lazy_static;
use std::collections::HashMap;
use std::io::{Write, stdin, stdout};
use std::process::exit;
use std::sync::Mutex;

fn math(str: &str) -> Result<f64, Err> {
    let mut tokens = parse(str)?;
    cal_recursive(&mut tokens)
}

fn main() {
    let mut line = String::new();

    loop {
        print!(">>> ");
        let _ = stdout().flush();
        let _ = stdin().read_line(&mut line);
        let trim = line.trim_start().trim_end();
        if trim == "exit" {
            exit(0);
        } else {
            match math(trim) {
                Ok(n) => println!("{n}"),
                Err(e) => match e {
                    Err::WrongChar => {
                        eprintln!("input the right numbers, variables or operators!!!")
                    }
                    Err::WrongBrackets => eprintln!("mismatched brackets!!!"),
                    Err::Cal => eprintln!("calculate wrong!!!"),
                    Err::UndefinedVar => eprintln!("variable doesn't exist!!!"),
                    _ => {}
                },
            }
            line.clear();
        }
    }
}

#[derive(Debug)]
enum Err {
    WrongChar,
    WrongBrackets,
    EmptyInput,
    Cal,
    Never,
    UndefinedVar,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Opt {
    Add,
    Sub,
    Mult,
    Div,
    Assign,
    Power,
    None,
}

#[derive(Debug, PartialEq)]
enum Bracket {
    Left,
    Right,
}

#[derive(Debug)]
struct OptAndWeight {
    opt: Opt,
    weight: u32,
}

#[derive(Debug)]
enum Tokens {
    Var(String),
    Opt(OptAndWeight),
    Bracket(Bracket),
}

const BRACKETS_OPTS: [char; 8] = ['(', ')', '+', '-', '*', '/', '=', '^'];

lazy_static! {
    static ref VARS: Mutex<HashMap<String, f64>> = Mutex::new(HashMap::new());
}

fn parse(str: &str) -> Result<Vec<Tokens>, Err> {
    let trim = str.trim_start().trim_end();

    if trim.is_empty() {
        return Err(Err::EmptyInput);
    }

    let mut no_opts_and_brackets = trim
        .split_terminator(BRACKETS_OPTS)
        .map(|s| s.trim_start().trim_end())
        .collect::<Vec<&str>>();

    let mut opts_and_brackets = trim
        .chars()
        .filter(|c| BRACKETS_OPTS.contains(c))
        .collect::<Vec<char>>();

    for n in no_opts_and_brackets.iter() {
        if n.contains(' ') {
            return Err(Err::WrongChar);
        }
    }

    no_opts_and_brackets.reverse();
    opts_and_brackets.reverse();

    // collect tokens
    let mut tokens = vec![];
    if trim.starts_with(BRACKETS_OPTS) {
        no_opts_and_brackets.pop();
        pop_opts_and_brackets(&mut opts_and_brackets, &mut tokens);
    }
    loop {
        if let Some(s) = no_opts_and_brackets.pop()
            && !s.is_empty()
        {
            if s.parse::<f64>().is_ok() {
                tokens.push(Tokens::Var(s.into()));
            } else if s.contains(|c: char| {
                !(c.is_ascii_digit()
                    || c.is_ascii_uppercase()
                    || c.is_ascii_lowercase()
                    || c == '_')
            }) || s.starts_with(|c: char| c.is_ascii_digit())
            {
                return Err(Err::WrongChar);
            } else {
                tokens.push(Tokens::Var(s.into()));
            }
        }
        pop_opts_and_brackets(&mut opts_and_brackets, &mut tokens);
        if no_opts_and_brackets.is_empty() && opts_and_brackets.is_empty() {
            break;
        }
    }

    deal_with_polarity(&mut tokens);

    Ok(tokens)
}

fn pop_opts_and_brackets(from: &mut Vec<char>, to: &mut Vec<Tokens>) {
    if let Some(c) = from.pop() {
        match c {
            '(' => to.push(Tokens::Bracket(Bracket::Left)),
            ')' => to.push(Tokens::Bracket(Bracket::Right)),
            '+' => to.push(Tokens::Opt(OptAndWeight {
                opt: Opt::Add,
                weight: 1,
            })),
            '-' => to.push(Tokens::Opt(OptAndWeight {
                opt: Opt::Sub,
                weight: 1,
            })),
            '*' => to.push(Tokens::Opt(OptAndWeight {
                opt: Opt::Mult,
                weight: 2,
            })),
            '/' => to.push(Tokens::Opt(OptAndWeight {
                opt: Opt::Div,
                weight: 2,
            })),
            '=' => to.push(Tokens::Opt(OptAndWeight {
                opt: Opt::Assign,
                weight: 0,
            })),
            '^' => to.push(Tokens::Opt(OptAndWeight {
                opt: Opt::Power,
                weight: 3,
            })),
            _ => {}
        }
    }
}

fn deal_with_polarity(tokens: &mut Vec<Tokens>) {
    tokens.reverse();

    let mut index_adds_subs = vec![];
    for (i, t) in tokens.iter().enumerate() {
        if let Tokens::Opt(o) = t
            && (o.opt == Opt::Add || o.opt == Opt::Sub)
        {
            index_adds_subs.push(i);
        }
    }
    index_adds_subs.reverse();

    for i in index_adds_subs {
        if let Some(t) = tokens.get(i + 1) {
            if let Tokens::Bracket(b) = t
                && *b == Bracket::Left
            {
                tokens.insert(i + 1, Tokens::Var("0".into()));
            } else if let Tokens::Opt(_) = t {
                tokens.insert(i + 1, Tokens::Var("0".into()));
            }
        } else {
            tokens.insert(i + 1, Tokens::Var("0".into()));
        }
    }

    tokens.reverse();
}

fn cal_recursive(tokens: &mut Vec<Tokens>) -> Result<f64, Err> {
    let mut var_left = "";
    let mut var_right = "";
    let mut opt = Opt::None;
    let mut flag_cal = false;
    let mut flag_remove_bracket = false;

    let mut i = 0;
    let pos = loop {
        if let Some(Tokens::Var(var_0)) = tokens.get(i)
            && let Some(Tokens::Opt(opt_1)) = tokens.get(i + 1)
            && let Some(Tokens::Var(var_1)) = tokens.get(i + 2)
        {
            if let Some(Tokens::Opt(opt_2)) = tokens.get(i + 3)
                && opt_2.weight > opt_1.weight
                && let Some(Tokens::Var(var_2)) = tokens.get(i + 4)
            {
                var_left = var_1;
                var_right = var_2;
                opt = opt_2.opt;
                i += 2;
                break Some(i);
            } else {
                var_left = var_0;
                var_right = var_1;
                opt = opt_1.opt;
                break Some(i);
            }
        }

        i += 1;
        if i >= tokens.len() - 1 {
            break None;
        }
    };

    if let Some(pos) = pos {
        let result = match opt {
            o @ (Opt::Add | Opt::Sub | Opt::Mult | Opt::Div | Opt::Power) => {
                let num_left;
                let num_right;
                let vars = VARS.lock().unwrap();

                if let Ok(n) = var_left.parse::<f64>() {
                    num_left = n;
                } else if let Some(n) = vars.get(var_left) {
                    num_left = *n;
                } else {
                    return Err(Err::UndefinedVar);
                }

                if let Ok(n) = var_right.parse::<f64>() {
                    num_right = n;
                } else if let Some(n) = vars.get(var_right) {
                    num_right = *n;
                } else {
                    return Err(Err::UndefinedVar);
                }
                match o {
                    Opt::Add => num_left + num_right,
                    Opt::Sub => num_left - num_right,
                    Opt::Div => num_left / num_right,
                    Opt::Mult => num_left * num_right,
                    Opt::Power => num_left.powf(num_right),
                    _ => {
                        return Err(Err::Never);
                    }
                }
            }
            Opt::Assign => {
                let num_right;
                let mut vars = VARS.lock().unwrap();

                if var_left.parse::<f64>().is_ok() {
                    return Err(Err::Cal);
                } else if let Ok(n) = var_right.parse::<f64>() {
                    num_right = n;
                } else if let Some(n) = vars.get(var_right) {
                    num_right = *n;
                } else {
                    return Err(Err::UndefinedVar);
                }

                vars.insert(var_left.to_string(), num_right);
                num_right
            }
            Opt::None => {
                return Err(Err::Never);
            }
        };
        if result.is_finite() {
            tokens[pos] = Tokens::Var(result.to_string());
            flag_cal = true;
        } else {
            return Err(Err::Cal);
        }
        tokens.remove(pos + 1);
        tokens.remove(pos + 1);
    }

    #[allow(clippy::never_loop)]
    loop {
        let left = tokens
            .iter()
            .enumerate()
            .filter(|(_, v)| matches!(v, Tokens::Bracket(Bracket::Left)))
            .map(|(i, _)| i)
            .collect::<Vec<usize>>();

        for p in left {
            if let Some(Tokens::Var(_)) = tokens.get(p + 1)
                && let Some(Tokens::Bracket(Bracket::Right)) = tokens.get(p + 2)
            {
                tokens.remove(p + 2);
                tokens.remove(p);
                flag_remove_bracket = true;
                continue;
            }
        }
        break;
    }

    if flag_cal || flag_remove_bracket {
        cal_recursive(tokens)
    } else if tokens.len() == 1
        && let Tokens::Var(v) = &tokens[0]
    {
        if let Ok(n) = v.parse::<f64>() {
            Ok(n)
        } else {
            let vars = VARS.lock().unwrap();
            if let Some(&n) = vars.get(v) {
                Ok(n)
            } else {
                Err(Err::UndefinedVar)
            }
        }
    } else {
        Err(Err::WrongBrackets)
    }
}
