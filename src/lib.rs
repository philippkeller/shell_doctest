extern crate rexpect;

use rexpect::session;
use rexpect::errors::*;

fn exp_bash(script:String, timeout:Option<u64>) {
    let res = || -> Result<()> {
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
                        Some(c) => format!("After running `{}`:", c),
                        None => "After running no command yet (!?):".into()
                    }
                })?;
            }
        }
        Ok(())
    }();
    if let Err(ref e) = res {
        eprintln!("error: {}", e);
        for e in e.iter().skip(1) {
            eprintln!("⤷ {}", e);
        }
        ::std::process::exit(1);
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_echo() {
        exp_bash("$ echo hans\n\
                  hans\
                  ".to_string(), Some(1000));
    }

    #[test]
    fn test_sleep() {
        exp_bash("$ sleep 10\n\
                  --> ^C\n\
                  $".to_string(), Some(1000));
    }
}