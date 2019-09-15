use std::collections::HashMap;
use std::path::Path;

#[derive(Debug)]
pub struct Config {
    pub nick: String,
    pub twtfile: String,
    pub twturl: String,
    pub pre_tweet_hook: String,
    pub post_tweet_hook: String,
    pub limit_timeline: i32,
    pub use_abs_time: bool,
    pub following: HashMap<String, String>,
}

impl Config {
    pub fn new(config_path: &Path) -> Config {
        use ini::Ini;

        let config = Ini::load_from_file(config_path).unwrap();
        let twtxt_config = config.section(Some("twtxt".to_owned())).unwrap();

        let following = config
            .section(Some("following".to_owned()))
            .unwrap()
            .to_owned();
        // Parse hook commands.
        let pre_tweet_hook = strfmt::strfmt(&twtxt_config["pre_tweet_hook"], twtxt_config).unwrap();
        let post_tweet_hook =
            strfmt::strfmt(&twtxt_config["post_tweet_hook"], twtxt_config).unwrap();

        Config {
            nick: twtxt_config["nick"].to_owned(),
            twtfile: twtxt_config["twtfile"].to_owned(),
            twturl: twtxt_config["twturl"].to_owned(),
            pre_tweet_hook: pre_tweet_hook,
            post_tweet_hook: post_tweet_hook,
            limit_timeline: twtxt_config["limit_timeline"].parse::<i32>().unwrap(),
            use_abs_time: twtxt_config["use_abs_time"].parse::<bool>().unwrap(),
            following: following,
        }
    }
}
