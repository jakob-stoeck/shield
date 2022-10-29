use std::env;
use std::process::{self, Stdio, Command};
use std::io::Write;

fn main() {
    // build config
    // sign into 1pass to get password
    // connect

    let args: Vec<String> = env::args().collect();
    let config = Config::build(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });
    run(config);
}

fn run(config: Config) {
    println!("Signing into 1Password");
    if let Err(e) = OnePassword::new() {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
    println!("Done signing into 1Password.");
    println!("{:?}", config);

    let pass = OnePassword::read(config.pass_path);

    println!("Connecting with AnyConnect …",);
    let vpn = AnyConnect {
        host: config.host,
        group: config.group,
        user: config.user,
    };

    vpn.connect(pass);
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
    fn new() -> Result<(), &'static str> {
        let output = Command::new("op").arg("signin").output().unwrap();

        if !output.status.success() {
            return Err("1Password biometric signin failed. To enable biometric unlock, navigate to  Developer Settings in the 1Password app and select \"Biometric Unlock for 1Password CLI\".");
        }
        Ok(())
    }

    fn read(path: String) -> String {
        let output = Command::new("op").arg("read").arg(path).output().unwrap();
        String::from_utf8(output.stdout).unwrap()
    }
}

#[derive(Debug)]
struct AnyConnect {
    host: String,
    group: String,
    user: String,
}
impl AnyConnect {
    fn connect(&self, pass: String) {
        let content = format!("{}\n{}\n{}", self.group, self.user, pass);

        let mut process = match Command::new("/opt/cisco/anyconnect/bin/vpn")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .args(["-s", "connect", &self.host])
        .spawn() {
            Err(e) => panic!("couldn’t spawn process {e}"),
            Ok(process) => process
        };

        match process.stdin.take().unwrap().write_all(content.as_bytes()) {
            Err(e) => panic!("couldn’t write to stdin: {e}"),
            _ => ()
        }

        let output = process.wait_with_output().expect("failed to read output");
        let out = String::from_utf8(output.stdout);
        println!("{:?}", out);
    }
}