use std::error::Error;

use app_dirs2::AppInfo;
use clap::{app_from_crate, Arg, ArgMatches, crate_authors, crate_description, crate_name, crate_version};

mod parser;

pub mod structs;
pub mod config;
pub mod processing;

const APP_INFO: AppInfo = AppInfo { name: "comic_sort", author: "Ondřej Vágner" };


pub fn get_matches() -> Result<ArgMatches<'static>, Box<dyn Error>> {
    let matches: ArgMatches = app_from_crate!()
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .help("Read from a specific config file")
            .default_value("config.yaml")
        )
        .get_matches();
    Ok(matches)
}

