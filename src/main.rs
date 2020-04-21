use json::{parse, JsonValue};
use std::{fs, io};
use std::path::PathBuf;
use shellexpand::tilde;

struct Mapping {
    title: String,
    dir: PathBuf,
    original_pattern: String,
    new_pattern: String,
    function: String,
}

impl Mapping {
    fn new() -> Mapping {
        Mapping {
            title: String::new(),
            dir: PathBuf::new(),
            original_pattern: String::new(),
            new_pattern: String::new(),
            function: String::new(),
        }
    }
}

fn main() -> io::Result<()> {
    let contents = fs::read_to_string("config.json")?;

    let config = parse(&contents[..])
        .expect("Couldn't read JSON file.");

    let root_dir = make_dir(&config["root"]);
    let download_dir = make_dir(&config["download"]);

    let files = fs::read_dir(&download_dir)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    let mappings: Vec<Mapping> = Vec::new();

    for member in config["mappings"].members() {
        let m = parse_mapping(member, &root_dir);
    }

    println!("{:?}", root_dir);
    println!("{}", download_dir);
    println!("{:?}", files);
    Ok(())
}


fn make_dir(src: &JsonValue) -> String {
    let mut dir = PathBuf::new();
    for item in src.members() {
        let item = item.as_str().unwrap();
        if item.ends_with(":") {
            dir.push(format!(r"{}\", item))
        } else {
            dir.push(item)
        }
    }
    tilde(dir.to_str().unwrap()).to_string()
}


fn parse_mapping(mapping: &JsonValue, root: &String) -> Mapping {
    let mut new_mapping = Mapping::new();
    new_mapping.title = mapping["title"].as_str().unwrap().to_string();

    let _root = PathBuf::from(root);
    let mut dir = PathBuf::new();

    if mapping["directory"].is_string() {
        dir = PathBuf::from(mapping["directory"].as_str().unwrap());
    } else {
        for member in mapping["directory"].members() {
            dir.push(member.as_str().unwrap());
        }
    }
    new_mapping.dir = _root.join(dir);

    println!("{}", new_mapping.title);
    println!("{:?}", new_mapping.dir);

    new_mapping
}