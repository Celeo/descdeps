use clap::{crate_version, load_yaml, App};
use std::{process, str::FromStr};
use strum_macros::{Display, EnumString};

#[derive(Debug, Display, EnumString)]
enum ProjectType {
    Rust,
    Python,
    Node,
}

fn get_project_type() -> ProjectType {
    unimplemented!()
}

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).version(crate_version!()).get_matches();

    let project_type: ProjectType = match matches.value_of("type") {
        Some(val) => match ProjectType::from_str(val) {
            Ok(ty) => ty,
            Err(_) => {
                println!("Error: unknown project type '{}'", val);
                process::exit(1);
            }
        },
        None => get_project_type(),
    };
    println!("Project type is {}", project_type);
}
