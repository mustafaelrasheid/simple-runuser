use clap::{Parser, ArgAction};

#[derive(Parser)]
#[command(name = "runuser")]
#[command(version = "0.1.0")]
#[command(author = "mustafaelrasheid")]
#[command(
    about = "simple runuser",
    long_about = None
)]
pub struct Cli {
    #[arg(short = 'u', long, value_name="user")]
    pub user: Option<String>,
    #[arg(
        short = 'p',
        long = "preserve_environment",
        visible_short_alias = 'm',
        action=ArgAction::SetTrue
    )]
    pub preserve_env: bool,
    #[arg(short = 'w', long = "whitelist_environment", value_name="list")]
    pub whitelist_env: Option<Vec<String>>,
    #[arg(short = 'g', long, value_name="group")]
    pub group: Option<String>,
    #[arg(short = 'G', long, value_name="supp-group")]
    pub supp_group: Option<Vec<String>>,
    #[arg(short = 'l', long, action=ArgAction::SetTrue)]
    pub login: bool,
    #[arg(short = 'c', long, value_name="command")]
    pub command: Option<String>,
    #[arg(long, value_name="command")]
    pub session_command: Option<String>,
    #[arg(short = 'f', long, action=ArgAction::SetTrue)]
    pub fast: bool,
    #[arg(short = 's', long, value_name="shell")]
    pub shell: Option<String>,
    #[arg(short = 'P', long, action=ArgAction::SetTrue)]
    pub pty: bool,
    pub rest: Vec<String>,
}
