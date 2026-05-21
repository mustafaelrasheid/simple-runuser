mod args;

use std::io::Error as IOError;
use std::fs::{read_to_string};
use std::env::args;
use std::process::exit;
use std::process::Command;
use std::os::unix::process::CommandExt;
use std::error::Error;
use nix::unistd::{setuid, setgid, Uid, Gid};
use clap::Parser;
use crate::args::Cli;

fn get_conf(filename: &str, query: &str)
-> Result<Option<Vec<String>>, IOError> {
    let file = read_to_string(filename)?;
    let line = file
        .lines()
        .find(|line| line.starts_with(&format!("{}:", &query)));

    match line {
        Some(line) => {
            let parts: Vec<String> = line
                .split(':')
                .map(|s| s.to_string())
                .collect();
            
            return Ok(Some(parts));
        }
        None => {
            return Ok(None);
        }
    }
}

fn get_user_info(username: &str)
-> Result<Option<(u32, u32, String, String)>, Box<dyn Error>> {
    let info = get_conf("/etc/passwd", username)?;
    
    match info {
        Some(parts) => {
            let [_, _,
                ref uid_str, ref gid_str, _,
                ref home_dir, ref entry_path]
                = parts[0..7] else {
                return Err(
                    "Invalid passwd format".into()
                );
            };
            let uid = uid_str.parse::<u32>()?;
            let gid = gid_str.parse::<u32>()?;

            return Ok(
                Some((
                    uid,
                    gid,
                    home_dir.to_string(),
                    entry_path.to_string()
                ))
            );
        }
        None => {
            return Ok(None);
        }
    }
}

fn run(
    path: &str,
    uid: u32,
    gid: u32,
    home_dir: &str,
    username: &str,
    args: &Vec<String>
) {
    let mut cmd = Command::new(path);
    
    cmd.env("HOME", home_dir);
    cmd.env("USER", username);
    cmd.env("LOGNAME", username);
    cmd.args(args);

    setgid(Gid::from_raw(gid))
        .unwrap_or_else(|e| {
            eprintln!("Failed to set Gid to {} due to {}", gid, e);
            exit(1);
        });
    setuid(Uid::from_raw(uid))
        .unwrap_or_else(|e| {
            eprintln!("Failed to set Uid to {} due to {}", uid, e);
            exit(1);
        });

    #[allow(unreachable_code)]{
        let err = cmd.exec();

        eprintln!("Failed to execv: {}", err);
        exit(1);
    }
}

fn main() {
    let cli = Cli::parse();
    
    if let Some(username) = cli.user {
        let path = cli.rest[0].clone();
        let command_args = cli.rest[1..].to_vec();
        
        if cli.rest.is_empty() {
            eprintln!("Incorrect usage");
            exit(1);
        }
        if let Some((uid, gid, home_dir, _entry_path)) = get_user_info(&username)
            .unwrap_or_else(|e| {
                eprintln!("Failed to get user info: {}", e);
                exit(1);
            }) {
            run(
                &path,
                uid,
                gid,
                &home_dir,
                &username,
                &command_args
            );
        } else {
            eprintln!("User doesn't exist");
            exit(1);
        }
    }
}
