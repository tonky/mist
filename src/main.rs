#![feature(let_chains)]
mod args;
mod cache;
mod clone;
mod comment;
mod install;
mod install_util;
mod list;
mod list_comments;
mod message;
mod progress;
mod remove;
mod search;
mod style;
mod update;
mod upgrade;
mod util;
mod whoami;

use args::{Cli, Commands};
use clap::Parser;
pub use rust_apt::util as apt_util;
use std::{
    env,
    fs::File,
    os::{linux::fs::MetadataExt, unix::fs::PermissionsExt},
};
use style::Colorize;
use which::which;

#[quit::main]
fn main() {
    // Make sure that this executable has the `setuid` flag set and is owned by
    // root. Parts of this program (intentionally) expect such behavior.
    let cmd_name = {
        let cmd = env::args().collect::<Vec<String>>().remove(0);
        if cmd.contains('/') {
            cmd
        } else {
            which(cmd).unwrap().into_os_string().into_string().unwrap()
        }
    };

    let cmd_metadata = File::open(cmd_name).unwrap().metadata().unwrap();

    // Make sure `root` owns the executable.
    if cmd_metadata.st_uid() != 0 {
        message::error("This executable needs to be owned by `root` in order to run.\n");
        quit::with_code(exitcode::USAGE);
    // Make sure the `setuid` bit flag is set. This appears to be third
    // digit in the six-digit long mode returned.
    } else if format!("{:o}", cmd_metadata.permissions().mode())
        .chars()
        .nth(2)
        .unwrap()
        .to_string()
        .parse::<u8>()
        .unwrap()
        < 4
    {
        message::error(
            "This executable needs to have the `setuid` bit flag set in order to run command.\n",
        );
        quit::with_code(exitcode::USAGE);
    }

    util::sudo::to_root();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Clone {
            package_name,
            mpr_url,
        } => clone::clone(package_name, &mpr_url.url),
        Commands::Comment {
            package_name,
            message,
            mpr_url,
            mpr_token,
        } => comment::comment(
            package_name,
            message,
            mpr_token.token.clone(),
            mpr_url.url.clone(),
        ),
        Commands::Install {
            package_names,
            mpr_url,
        } => {
            util::sudo::check_perms();

            if is_running_as_sudo() {
                return;
            }

            install::install(package_names, mpr_url.url.clone())
        }
        Commands::List {
            package_names,
            mode,
            mpr_url,
            name_only,
            installed_only,
        } => println!(
            "{}",
            list::list(
                package_names,
                &mpr_url.url,
                mode,
                *name_only,
                *installed_only
            )
        ),
        Commands::ListComments {
            package_name,
            mpr_url,
            paging,
        } => list_comments::list_comments(package_name, &mpr_url.url, paging),
        Commands::Remove {
            package_names,
            mpr_url,
            purge,
            autoremove,
        } => {
            util::sudo::check_perms();
            remove::remove(package_names, &mpr_url.url, *purge, *autoremove)
        }
        Commands::Search {
            query,
            mode,
            mpr_url,
            name_only,
            installed_only,
        } => println!(
            "{}",
            search::search(query, &mpr_url.url, mode, *name_only, *installed_only)
        ),
        Commands::Update { mpr_url } => {
            util::sudo::check_perms();
            update::update(&mpr_url.url)
        }
        Commands::Upgrade { mpr_url, mode } => {
            util::sudo::check_perms();

            if is_running_as_sudo() {
                return;
            }

            upgrade::upgrade(&mpr_url.url, mode)
        }
        Commands::Whoami { mpr_url, mpr_token } => {
            whoami::whoami(mpr_token.token.clone(), mpr_url.url.clone())
        }
    };
}

fn is_running_as_sudo() -> bool {
    if *util::sudo::NORMAL_UID == 0 {
        message::error(&format!(
            "This command cannot be ran as root, as it needs to call '{}', which is required to run under a non-root user.\n",
            "makedeb".bold().green()
        ));

        true
    } else {
        false
    }
}
