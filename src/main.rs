use std::env;
use std::io::{self, Write};
use std::process::{self, Command, Stdio};

fn main() {
    // build config
    // sign into 1pass to get password
    // connect

    let args: Vec<String> = env::args().collect();
    let config = Config::build(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });
    if let Err(e) = run(config) {
        eprintln!("{e}");
        process::exit(1);
    }
}

fn run(config: Config) -> io::Result<()> {
    println!("Signing into 1Password");
    OnePassword::new()?;
    let pass = OnePassword::read(config.pass_path)?;

    println!("Connecting with AnyConnect …",);
    let vpn = AnyConnect {
        host: config.host,
        group: config.group,
        user: config.user,
    };
    let out = vpn.connect(pass)?;
    println!("Connected: {out}");

    Ok(())
}

#[derive(Debug)]
struct Config {
    host: String,
    group: String,
    user: String,
    pass_path: String,
}
impl Config {
    fn build(args: &[String]) -> Result<Config, &str> {
        if args.len() < 4 {
            return Err("Not enough arguments.");
        }
        let host = args[1].clone();
        let group = args[2].clone();
        let user = args[3].clone();
        let pass_path = args[4].clone();
        Ok(Config { host, group, user, pass_path })
    }
}

struct OnePassword {}
impl OnePassword {
    fn new() -> io::Result<()> {
        let status = Command::new("op").arg("signin").status().expect("failed to execute command 'op'");
        if !status.success() {
            panic!("1Password biometric signin failed. To enable biometric unlock, navigate to  Developer Settings in the 1Password app and select \"Biometric Unlock for 1Password CLI\".");
        }
        Ok(())
    }

    fn read(path: String) -> io::Result<String> {
        let output = Command::new("op").arg("read").arg(path).output()?;
        // we know this value must be utf8, so just use expect()
        Ok(String::from_utf8(output.stdout).expect("invalid utf8"))
    }
}

#[derive(Debug)]
struct AnyConnect {
    host: String,
    group: String,
    user: String,
}
impl AnyConnect {
    fn connect(&self, pass: String) -> io::Result<String> {
        let mut process = Command::new("/opt/cisco/anyconnect/bin/vpn")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .args(["-s", "connect", &self.host])
        .spawn()?;

        let content = format!("{}\n{}\n{}", self.group, self.user, pass);
        let mut stdin = process.stdin.take().unwrap();
        stdin.write_all(content.as_bytes())?;

        let output = process.wait_with_output()?;
        Ok(String::from_utf8(output.stdout).expect("invalid utf8"))
    }
}