use std::fs;
use std::path::PathBuf;

use app_dirs2::{app_dir, AppDataType, AppInfo};
use serde_yaml::from_str;

use crate::config::Config;
use crate::processor::process_file;

mod config;
mod processor;

const APP_INFO: AppInfo = AppInfo { name: "comic_sort", author: "Ondřej Vágner" };


fn main() {
    let file: Option<PathBuf> = Some({
        let name: PathBuf = PathBuf::from("config.yaml");

        if !&name.exists() {
            let folder: PathBuf = app_dir(AppDataType::UserConfig, &APP_INFO, "/").unwrap();
            folder.join(&name)
        } else {
            name
        }
    });

    let mut config: Config = from_str(
    fs::read_to_string(file.unwrap().to_str().unwrap()).unwrap().as_str())
        .expect("Couldn't read YAML file");

    config.files = config.get_files().expect("Couldn't read the download folder");

    for mapping in &mut config.mappings {
        let _ = mapping.make_patterns();
    }

    for file in &config.files {
        process_file(
            file,
            &config
        ).unwrap();
    }
}
