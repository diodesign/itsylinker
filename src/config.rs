/* Parse the configuration file format
 * 
 * 
 * (c) Chris Williams, 2021.
 *
 * See LICENSE for usage and copying.
 */

use serde_derive::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
pub struct Config
{
    output: Output,
    section: Option<BTreeMap<String, Section>>
}

impl Config
{
    pub fn get_entry(&self) -> &String { &self.output.entry }
}

#[derive(Deserialize)]
struct Output
{
    entry: String,
    sections: Vec<String>,
    start_symbol: Option<String>,
    end_symbol: Option<String>,
    alignment: usize,
    dynamic_relocation: bool
}

#[derive(Deserialize)]
struct Section
{
    start_symbol: Option<String>,
    end_symbol: Option<String>,
    alignment: usize
}

/* load the given file into memory and parse it, returning a config structure */
pub fn parse_config(filename: &String) -> Config
{
    let config_contents = match std::fs::read_to_string(filename)
    {
        Ok(c) => c,
        Err(e) =>
        {
            eprintln!("Can't read configuration file {}: {}", filename, e);
            std::process::exit(1);
        }
    };

    match toml::from_str(config_contents.as_str())
    {
        Ok(c) => c,
        Err(e) =>
        {
            eprintln!("Can't parse configutation file {}: {}", filename, e);
            std::process::exit(1);
        }
    }
}
