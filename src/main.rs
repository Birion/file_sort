use anyhow::Result;
use human_panic::setup_panic;

use file_sort::prelude::*;

fn main() -> Result<()> {
    setup_panic!();
    perform_processing_based_on_configuration(get_configuration_file_option()?)
}