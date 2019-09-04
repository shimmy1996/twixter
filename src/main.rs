use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;

use clap::{crate_version, App, Arg, SubCommand};

#[derive(Debug)]
struct Config {
    nick: String,
    twtfile: String,
    twturl: String,
    scp_addr: String,
    scp_port: String,
    following: HashMap<String, String>,
}

impl Config {
    fn new(config_path: &Path) -> Config {
        use ini::Ini;

        let config = Ini::load_from_file(config_path).unwrap();
        let twtxt_config = config.section(Some("twtxt".to_owned())).unwrap();

        let mut following = config
            .section(Some("following".to_owned()))
            .unwrap()
            .to_owned();
        // Always follow oneself.
        *following
            .entry(twtxt_config["nick"].to_owned())
            .or_default() = twtxt_config["twturl"].to_owned();

        Config {
            nick: twtxt_config["nick"].to_owned(),
            twtfile: twtxt_config["twtfile"].to_owned(),
            twturl: twtxt_config["twturl"].to_owned(),
            scp_addr: twtxt_config["scp_addr"].to_owned(),
            scp_port: twtxt_config["scp_port"].to_owned(),
            following: following,
        }
    }
}

fn main() {
    let command = App::new("twixter")
        .version(crate_version!())
        .about("A client for twtxt, microblog for hackers")
        .arg(
            Arg::with_name("config_dir")
                .short("c")
                .long("config")
                .value_name("PATH")
                .help("Specifies a custom config file location")
                .takes_value(true),
        )
        .subcommand(
            SubCommand::with_name("tweet")
                .version(crate_version!())
                .about("Append a new tweet to your twtxt file")
                .arg(Arg::with_name("content").multiple(true)),
        )
        .subcommand(SubCommand::with_name("timeline").about("Retrieves your timeline"))
        .subcommand(
            SubCommand::with_name("follow")
                .version(crate_version!())
                .about("Adds a new source to your followings")
                .arg(
                    Arg::with_name("nick")
                        .required(true)
                        .value_name("NICK")
                        .help("Specifies nickname to store source with"),
                )
                .arg(
                    Arg::with_name("url")
                        .required(true)
                        .value_name("URL")
                        .help("Specifies source url"),
                ),
        )
        .get_matches();

    // Source config.
    let mut config_dir = command
        .args
        .get("config_dir")
        .and_then(|matched_arg| Some(Path::new(&matched_arg.vals[0]).to_path_buf()))
        .unwrap_or({
            let mut config_dir = dirs::config_dir().unwrap();
            config_dir.push("twixter");
            config_dir
        });
    config_dir.push("config");

    let config = Config::new(&config_dir);

    // Check if twtfile exists and create one if necessary.
    let twtfile_path = Path::new(&config.twtfile).parent().unwrap();
    if !twtfile_path.exists() {
        std::fs::create_dir_all(twtfile_path).unwrap();
    }

    // Fetch current twtfile.
    fetch(&config);

    // Add user post.
    let content = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
    if content == "" {
        eprintln!("Error: post content must not be empty");
    } else {
        let post = compose(content);

        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&config.twtfile)
            .unwrap();

        file.write(post.as_bytes())
            .expect("Unable to write new post");
    }

    // Publish to remote.
    publish(&config);
}

fn fetch(config: &Config) {
    // Uses scp to fetch latest twtfile.
    Command::new("scp")
        .args(&["-P", &config.scp_port, &config.scp_addr, &config.twtfile])
        .output()
        .expect("Failed to pull!");
}

fn publish(config: &Config) {
    // Publishes twtfile to remote.
    Command::new("scp")
        .args(&["-P", &config.scp_port, &config.twtfile, &config.scp_addr])
        .output()
        .expect("Failed to publish!");
}

fn compose(content: String) -> String {
    // Formats given content into twtxt format by adding datetime.
    use chrono::{Local, SecondsFormat};
    let timestamp = Local::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let mut post = String::new();
    post.push_str(&timestamp);
    post.push('\t');
    post.push_str(&content);
    post.push('\n');
    post
}
