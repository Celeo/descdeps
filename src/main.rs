use clap::{crate_version, load_yaml, App, ArgMatches};
use log::{debug, error, info, LevelFilter};
use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Config, Logger, Root},
    encode::pattern::PatternEncoder,
};
use std::{fs, path::PathBuf, process, str::FromStr};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

mod drivers;
mod node_driver;
mod rust_driver;

use drivers::Driver;
use node_driver::NodeDriver;
use rust_driver::RustDriver;

fn setup_logger(enable_debug: bool) {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} {h({l})} {m}{n}")))
        .build();
    let minimum_level = if enable_debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .logger(
            Logger::builder()
                .appender("stdout")
                .additive(false)
                .build("descdeps", minimum_level),
        )
        .build(Root::builder().appender("stdout").build(LevelFilter::Warn))
        .unwrap();
    log4rs::init_config(config).expect("Could not set log4rs configuration");
}

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
    fn get_path_matchers(self) -> Vec<PathBuf> {
        match self {
            ProjectType::Rust => vec![["Cargo.toml"].iter().collect()],
            ProjectType::Node => vec![["package.json"].iter().collect()],
            ProjectType::Python => vec![["requirements.txt"].iter().collect()],
        }
    }

    fn driver(self, user_agent: &str) -> Box<dyn Driver> {
        match self {
            ProjectType::Rust => Box::from(RustDriver::new(user_agent)),
            ProjectType::Node => Box::from(NodeDriver::new(user_agent)),
            ProjectType::Python => {
                error!("Support for this language is not yet available");
                process::exit(0);
            }
        }
    }
}

fn match_project_type() -> Option<(ProjectType, PathBuf)> {
    for project_type in ProjectType::iter() {
        if let Some(pt) = match_single_project_type(project_type) {
            return Some(pt);
        }
    }
    None
}

fn match_single_project_type(project_type: ProjectType) -> Option<(ProjectType, PathBuf)> {
    for path in project_type.get_path_matchers() {
        debug!("Checking path {:?} for type {}", path, project_type);
        if path.exists() {
            return Some((project_type, path));
        }
    }
    None
}

fn command_main(matches: &ArgMatches, user_agent: &str) {
    debug!("Determining project type");
    let (project_type, path): (ProjectType, PathBuf) = match matches.value_of("type") {
        Some(override_value) => {
            let project_type = match ProjectType::from_str(override_value) {
                Ok(pt) => pt,
                Err(_) => {
                    error!(
                        "Could not match '{}' to a known project type",
                        override_value
                    );
                    process::exit(1);
                }
            };
            let path = match match_single_project_type(project_type) {
                Some(pair) => pair.1,
                None => {
                    error!("Could not find matching dependencies file for overridden project type '{}'", override_value);
                    process::exit(1);
                }
            };
            (project_type, path)
        }
        None => match match_project_type() {
            Some(pair_found) => pair_found,
            None => {
                error!("Could not determine project type");
                process::exit(1);
            }
        },
    };
    debug!("Project type is {}", project_type);

    debug!("Constructing driver");
    let driver = project_type.driver(user_agent);

    debug!("Reading dependency file");
    let content = match fs::read_to_string(path.clone()) {
        Ok(c) => c,
        Err(e) => {
            debug!("Could not read file '{}': {}", path.display(), e);
            error!("Could not read dependencies file '{}'", path.display());
            process::exit(1);
        }
    };

    debug!("Getting info from driver");
    info!("Fetching descriptions");
    driver.print_info(&content);
}

fn command_list(_matches: &ArgMatches) {
    let supported = ProjectType::iter()
        .map(|pt| pt.to_string())
        .collect::<Vec<String>>()
        .join(", ");
    info!("Supported languages: {}", supported);
}

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).version(crate_version!()).get_matches();
    let debug_enabled = matches.is_present("debug");
    setup_logger(debug_enabled);
    let user_agent = match matches.value_of("agent") {
        Some(val) => val,
        None => "",
    };
    debug!("User agent set to: {}", user_agent);

    if let Some(subcommand) = matches.subcommand_name() {
        match subcommand {
            "list" => command_list(&matches),
            _ => {
                error!("Unsupported subcommand");
                process::exit(1);
            }
        }
    } else {
        command_main(&matches, user_agent);
    };
}
