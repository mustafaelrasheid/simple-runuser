use std::io;
use std::io::{Error, ErrorKind};
use std::io::ErrorKind::NotFound;
use std::fs::{read_to_string};
use std::env::args;
use std::process::exit;
use std::process::Command;
use std::os::unix::process::CommandExt;
use nix::unistd::{setuid, setgid, Uid, Gid};

fn get_conf(filename: &str, query: &str) -> io::Result<Vec<String>> {
    let line = read_to_string(filename)?
        .lines()
        .find(|lines| lines.starts_with(query))
        .ok_or_else(|| {
            let err_msg = format!("Invalid {} format", filename);
            Error::new(NotFound, &*err_msg)
        })?
        .to_string();

    let parts: Vec<String> = line.split(':').map(|s| s.to_string()).collect();
    return Ok(parts);
}


fn get_user_info(username: &str) -> io::Result<(u32, u32, String, String)> {
    let query = format!("{}:", username);
    let parts = get_conf("/etc/passwd", &query)?;
    let [_, _, ref uid_str, ref gid_str, _, ref home_dir, ref entry_path] = parts[0..7] else {
        return Err(
            Error::new(
                ErrorKind::Other,
                "Invalid passwd format"));
    };

    let uid = uid_str.parse::<u32>()
        .map_err(|_| Error::new(ErrorKind::Other, "Invalid UID"))?;
    let gid = gid_str.parse::<u32>()
        .map_err(|_| Error::new(ErrorKind::Other, "Invalid GID"))?;

    return Ok((uid, gid, home_dir.to_string(), entry_path.to_string()));
}

fn run(path: &str, uid: u32, gid: u32, home_dir: &str, username: &str, args: &Vec<String>) {
    let mut cmd = Command::new(path);
    
    cmd.env("HOME", home_dir);
    cmd.env("USER", username);
    cmd.env("LOGNAME", username);
    cmd.args(args);

    setgid(Gid::from_raw(gid)).unwrap_or_else(|e| {
        eprintln!("Failed to set Gid to {} due to {}", gid, e);
        exit(1);
    });
    setuid(Uid::from_raw(uid)).unwrap_or_else(|e| {
        eprintln!("Failed to set Uid to {} due to {}", uid, e);
        exit(1);
    });
    #[allow(unreachable_code)]{
        let _ = cmd.exec();
    }
}

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() < 4 {
        eprintln!("Usage: runuser -u username command [args...]");
        exit(1);
    }
    if args[1] != "-u" {
        eprintln!("Usage: runuser -u username command [args...]");
        exit(1);
    }
    let username = args[2].clone();
    let path = args[3].clone();
    let command_args = &args[4..].to_vec();
    let (uid, gid, home_dir, _entry_path) = get_user_info(&username).unwrap();

    run(&path, uid, gid, &home_dir, &username, command_args);
}
