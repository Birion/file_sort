use std::error::Error;
use std::path::PathBuf;

use app_dirs2::{app_dir, AppDataType};

use super::APP_INFO;

pub fn read(config: PathBuf) -> Result<PathBuf, Box<dyn Error>> {
    if !&config.exists() {
        let folder: PathBuf = app_dir(AppDataType::UserConfig, &APP_INFO, "/").unwrap();
        Ok(folder.join(&config))
    } else {
        Ok(config)
    }
}