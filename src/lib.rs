extern crate rexpect;

use rexpect::session;
use rexpect::errors::*;
use std::fs::File;
use std::io::prelude::*;

/// interpret shell_doctest script and halt/return on first error
fn exp_bash(filename:&str, timeout:Option<u64>) -> Result<()> {
    let mut file = File::open(filename).chain_err(|| "cannot open exp bash script")?;
    let mut script = String::new();
    file.read_to_string(&mut script).chain_err(|| "cannot read from exp bash script")?;
    exp_bash_string(script, timeout)?;
    Ok(())
}

/// run exp_bash and print error
pub fn run_test(filename:&str) {
    let res = exp_bash(filename, Some(500));
    if let Err(ref e) = res {
        eprintln!("error: {}", e);
        for e in e.iter().skip(1) {
            eprintln!("⤷ {}", e);
        }
        ::std::process::exit(1);
    }
}

fn exp_bash_string(script:String, timeout:Option<u64>) -> Result<()> {
    let mut p = session::spawn_bash(timeout)?;
    let mut last_cmd = None;
    for line in script.lines() {
        if line.starts_with('$') {
            if last_cmd != None {
                p.wait_for_prompt()?;
            }
            let cmd = line[1..].trim();
            p.execute(cmd)?;
            last_cmd = Some(cmd);
        } else if line.starts_with("-->") {
            let input:&str = line[4..].trim();
            if input.starts_with('^') && input.len() == 2 {
                p.send_control(input.chars().nth(1).unwrap())?;
            } else {
                p.send_line(input)?;
            }
        } else if line.trim().len() == 0 {
            // empty line, do nothing
        } else {
            p.exp_regex(&line).chain_err(|| {
                match last_cmd {
                    Some(c) => format!("After running `{}`", c),
                    None => "After running no command yet (!?)".into()
                }
            })?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_echo() {
        exp_bash_string("$ echo hans\n\
                  hans\
                  ".to_string(), Some(1000)).expect("test_echo failed");
    }

    #[test]
    fn test_sleep() {
        exp_bash_string("$ sleep 10\n\
                  --> ^C\n\
                  $".to_string(), Some(1000)).expect("test_sleep failed");
    }

    #[test]
    fn test_sleep2() {
        exp_bash_string("$ sleep 999\n\
                  --> ^Z\n\
                  $ echo hans\n\
                  hans\n\
                  $ fg\n\
                  --> ^C\n\
                  $\
                  ".to_string(), Some(1000)).expect("test_sleep2 failed");
    }

    #[test]
    fn test_sleep3() {
        run_test("tests/scripts/sleep_test");
    }

}