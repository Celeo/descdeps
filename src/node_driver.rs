use crate::drivers::{name_desc_to_table, Driver};
use failure::{format_err, Error};
use log::debug;
use rayon::prelude::*;
use reqwest::{header, Client, Url};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct NodeDriver {
    base_url: Url,
    client: Client,
}

impl Driver for NodeDriver {
    fn print_info(&self, data: &str) {
        let pairs = get_deps_from_package_json(data).unwrap();
        let parts: Vec<(String, String)> = pairs
            .par_iter()
            .map(|name| {
                let description = get_module_description(&self.client, &self.base_url, name);
                (name.to_owned(), description)
            })
            .collect();
        let table = name_desc_to_table(&parts);
        table.print_tty(false);
    }
}

impl NodeDriver {
    pub fn new(user_agent: &str) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_str(user_agent).unwrap(),
        );
        Self {
            base_url: Url::parse("https://registry.npmjs.org/").unwrap(),
            client: Client::builder().default_headers(headers).build().unwrap(),
        }
    }
}

fn get_deps_from_package_json(content: &str) -> Result<Vec<String>, Error> {
    debug!("Parsing file as JSON");
    let json: Value = serde_json::from_str(content)?;
    let dep_map = match json["dependencies"].as_object() {
        Some(map) => map,
        None => {
            debug!("Could not get dependencies from file");
            return Ok(vec![]);
        }
    };
    Ok(dep_map.iter().map(|pair| pair.0.to_owned()).collect())
}

fn get_module_info(client: &Client, base_url: &Url, name: &str) -> Result<Value, Error> {
    let url = format!("{}{}", base_url, name);
    debug!("Making GET call to: {}", url);
    let mut resp = client.get(&url).send()?;
    if !resp.status().is_success() {
        return Err(format_err!(
            "Bad status {} from registry.npmjs.org API",
            resp.status()
        ));
    }
    let json: Value = resp.json()?;
    Ok(json)
}

fn get_module_description(client: &Client, base_url: &Url, name: &str) -> String {
    let json: Value = match get_module_info(client, base_url, name) {
        Ok(j) => j,
        Err(e) => {
            debug!("Error getting module info: {}", e);
            return String::from("-- unknown --");
        }
    };
    let description = match json["description"].as_str() {
        Some(d) => d.trim(),
        None => {
            debug!("Could not find JSON path in data");
            "-- unknown --"
        }
    };
    description.to_owned()
}
