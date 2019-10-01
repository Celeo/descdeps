use clap::{crate_version, load_yaml, App, ArgMatches};
use colored::Colorize;
use std::{path::PathBuf, process, str::FromStr};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

#[derive(Debug, Display, EnumString, EnumIter, Copy, Clone)]
enum ProjectType {
    Rust,
    Python,
    Node,
}

impl ProjectType {
    fn path_matchers(self) -> Vec<PathBuf> {
        match self {
            ProjectType::Rust => vec![["Cargo.toml"].iter().collect()],
            ProjectType::Python => vec![["requirements.txt"].iter().collect()],
            ProjectType::Node => vec![["package.json"].iter().collect()],
        }
    }
}

fn debug(matches: &ArgMatches, message: &str) {
    if matches.is_present("debug") {
        println!("{}: {}", "DEBUG".cyan(), message);
    }
}

fn get_project_type(matches: &ArgMatches) -> Option<ProjectType> {
    for project_type in ProjectType::iter() {
        for path in project_type.path_matchers() {
            debug(
                &matches,
                &format!("Checking path {:?} for type {}", path, project_type),
            );
            if path.exists() {
                return Some(project_type);
            }
        }
    }
    None
}

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).version(crate_version!()).get_matches();
    debug(&matches, "Starting");

    debug(&matches, "Determining project type");
    let project_type: ProjectType = match matches.value_of("type") {
        Some(val) => match ProjectType::from_str(val) {
            Ok(ty) => ty,
            Err(_) => {
                println!("{} unknown project type '{}'", "Error".red(), val);
                process::exit(1);
            }
        },
        None => match get_project_type(&matches) {
            Some(ty) => ty,
            None => {
                println!("{} could not determine project version", "Error".red());
                return;
            }
        },
    };
    println!("Project type is {}", project_type);
}
