mod args;

use std::env::var;
use std::io::Error as IOError;
use std::fs::{read_to_string};
use std::process::exit;
use std::process::Command;
use std::os::unix::process::CommandExt;
use std::error::Error;
use nix::unistd::{setuid, setgid, setgroups, Uid, Gid};
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
                ref home_dir, ref shell_path]
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
                    shell_path.to_string()
                ))
            );
        }
        None => {
            return Ok(None);
        }
    }
}

fn get_group_info(groupname: &str) -> Result<Option<u32>, Box<dyn Error>> {
    let info = get_conf("/etc/group", groupname)?;
    
    match info {
        Some(parts) => {
            let gid = parts
                .get(2)
                .ok_or("Invalid group format")?
                .parse::<u32>()?;

            return Ok(Some(gid));
        },
        None => {
            return Ok(None);
        }
    }
}

fn run(
    path: &str,
    uid: u32,
    gid: u32,
    supp_gids: &Vec<Gid>,
    home_dir: &str,
    shell_path: &str,
    username: &str,
    args: &Vec<String>,
    preserve_env: bool,
    whitelist_env: &Vec<String>,
) {
    const ROOT_PATH: &str = 
        "/usr/local/sbin:/usr/local/bin:/sbin:/bin:/usr/sbin:/usr/bin";
    let mut cmd = Command::new(path);
    
    if !preserve_env {
        cmd.env_clear();
        match username {
            "root" => {
                cmd.env("PATH", ROOT_PATH);
            },
            _ => {
                cmd.env("PATH", "/usr/local/bin:/usr/bin:/bin");
            }
        }
    }
    cmd.env("HOME", home_dir);
    cmd.env("USER", username);
    cmd.env("LOGNAME", username);
    cmd.env("SHELL", shell_path);
    for item in whitelist_env {
        if let Ok(val) = var(item) {
            cmd.env(item, val);
        }
    }
    cmd.args(args);
    setgroups(supp_gids.as_slice())
        .unwrap_or_else(|e| {
            eprintln!("Failed to set supp Gid to {} due to {}", uid, e);
            exit(1);
        });
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
    let supp_gids = if let Some(supp_groups) = cli.supp_group {
        let mut supp_gids = Vec::new();

        for supp_group in supp_groups {
            supp_gids.push(
                Gid::from_raw(
                    get_group_info(&supp_group)
                        .unwrap_or_else(|e| {
                            eprintln!("Failed to get supp group info: {}", e);
                            exit(1);
                        })
                        .unwrap_or_else(|| {
                            eprintln!("Supp group doesn't exist");
                            exit(1);
                        })
                )
            );
        }

        Some(supp_gids)
    } else { None };
    let overwritten_gid = if let Some(overwritten_group) = cli.group {
        Some(
            get_group_info(&overwritten_group)
                .unwrap_or_else(|e| {
                    eprintln!("Failed to get group info: {}", e);
                    exit(1);
                })
                .unwrap_or_else(|| {
                    eprintln!("Group doesn't exist");
                    exit(1);
                })
        )
    } else { None };
        
    if cli.rest.is_empty() {
        eprintln!("Incorrect usage");
        exit(1);
    }
    let (username, path, command_args) = match cli.user {
        Some(username) => {
            let path = cli.rest[0].clone();
            let command_args = cli.rest[1..].to_vec();

            (username, Some(path), command_args)
        },
        None => {
            let mut rest = cli.rest.clone();
            let mut hyphen: bool = false;
            if &rest[0] == "-" {
                rest.remove(0);
                hyphen = true;
            }
            if rest.is_empty() {
                eprintln!("Incorrect usage");
                exit(1);
            }
            let username = rest[0].clone();
            let mut command_args = rest[1..].to_vec();

            if cli.login || hyphen {
                command_args.insert(0, "-l".to_string());
            }
            if cli.fast {
                command_args.insert(0 ,"-f".to_string());
            }
            if let Some(cmd) = &cli.command {
                command_args.push("-c".to_string());
                command_args.push(cmd.clone());
            }

            (username, cli.shell, command_args)
        }
    };
    let (uid, gid, home_dir, shell_path) = get_user_info(&username)
        .unwrap_or_else(|e| {
            eprintln!("Failed to get user info: {}", e);
            exit(1);
        }).unwrap_or_else(|| {
        eprintln!("User doesn't exist");
        exit(1);
    });

    run(
        &path.unwrap_or(shell_path.clone()),
        uid,
        overwritten_gid.unwrap_or(gid),
        &supp_gids.unwrap_or(Vec::new()),
        &home_dir,
        &shell_path,
        &username,
        &command_args,
        cli.preserve_env,
        &cli.whitelist_env.unwrap_or(Vec::new())
    );
}
