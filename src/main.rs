use clap::{crate_version, load_yaml, App};
use colored::Colorize;
use std::{path::PathBuf, process, str::FromStr};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

mod rust_driver;
mod traits;

use rust_driver::RustDriver;
use traits::Driver;

#[derive(Debug, Display, EnumString, EnumIter, Copy, Clone)]
enum ProjectType {
    #[strum(serialize = "rust")]
    Rust,
    #[strum(serialize = "python")]
    Python,
    #[strum(serialize = "node")]
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

    fn driver(self, user_agent: &str) -> Box<dyn Driver> {
        match self {
            ProjectType::Rust => Box::from(RustDriver::new(user_agent)),
            ProjectType::Python => unimplemented!(),
            ProjectType::Node => unimplemented!(),
        }
    }
}

fn debug(enabled: bool, message: &str) {
    if enabled {
        println!("{}: {}", "DEBUG".cyan(), message);
    }
}

fn get_project_type(debug_enabled: bool) -> Option<ProjectType> {
    for project_type in ProjectType::iter() {
        for path in project_type.path_matchers() {
            debug(
                debug_enabled,
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
    let debug_enabled = matches.is_present("debug");
    let user_agent = match matches.value_of("agent") {
        Some(val) => val,
        None => "",
    };

    debug(debug_enabled, "Starting");

    debug(debug_enabled, "Determining project type");
    let project_type: ProjectType = match matches.value_of("type") {
        Some(val) => match ProjectType::from_str(val) {
            Ok(ty) => ty,
            Err(_) => {
                println!("{} unknown project type '{}'", "Error".red(), val);
                process::exit(1);
            }
        },
        None => match get_project_type(debug_enabled) {
            Some(ty) => ty,
            None => {
                println!("{} could not determine project version", "Error".red());
                return;
            }
        },
    };
    println!("Project type is {}", project_type);

    let driver = project_type.driver(user_agent);
}
