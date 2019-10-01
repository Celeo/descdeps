use crate::traits::Driver;
use failure::{format_err, Error};
use log::debug;
use prettytable::{cell, row, Table};
use rayon::prelude::*;
use reqwest::{header, Client, Url};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct RustDriver {
    base_url: Url,
    client: Client,
}

impl Driver for RustDriver {
    fn print_info(&self, data: &str) {
        let mut deps = vec![];
        let mut in_deps = false;
        for line in data.split('\n') {
            if line.trim() == "[dependencies]" {
                in_deps = true;
                continue;
            }
            if in_deps && line.trim().starts_with('[') {
                break;
            }
            if !in_deps {
                continue;
            }
            if let Some(split_index) = line.find('=') {
                deps.push(line.split_at(split_index).0.trim());
            }
        }
        debug!("Found {} dependencies", deps.len());
        let parts: Vec<(String, String)> = deps
            .par_iter()
            .map(|&name| {
                let description = get_crate_description(&self.client, &self.base_url, name);
                (name.to_owned(), description)
            })
            .collect();
        let table = info_to_table(&parts);
        table.print_tty(false);
    }
}

impl RustDriver {
    pub fn new(user_agent: &str) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_str(user_agent).unwrap(),
        );
        Self {
            base_url: Url::parse("https://crates.io/api/v1/crates/").unwrap(),
            client: Client::builder().default_headers(headers).build().unwrap(),
        }
    }
}

fn get_crate_info(client: &Client, base_url: &Url, name: &str) -> Result<Value, Error> {
    let url = format!("{}{}", base_url, name);
    debug!("Making GET call to: {}", url);
    let mut resp = client.get(&url).send()?;
    if !resp.status().is_success() {
        return Err(format_err!(
            "Bad status {} from crates.io API",
            resp.status()
        ));
    }
    let json: Value = resp.json()?;
    Ok(json)
}

fn get_crate_description(client: &Client, base_url: &Url, name: &str) -> String {
    let json: Value = match get_crate_info(client, base_url, name) {
        Ok(j) => j,
        Err(e) => {
            debug!("Error getting crate info: {}", e);
            return String::from("-- unknown --");
        }
    };
    let description = match json["crate"]["description"].as_str() {
        Some(d) => d.trim(),
        None => {
            debug!("Could not find JSON path in data");
            "-- unknown --"
        }
    };
    description.to_owned()
}

fn info_to_table(parts: &[(String, String)]) -> Table {
    let mut table = Table::new();
    table.set_titles(row![bFg->"Name", bFg->"Description"]);
    for part in parts {
        table.add_row(row![Fc->&part.0, &part.1]);
    }
    table
}
