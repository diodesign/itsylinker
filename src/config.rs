/* Parse the configuration file format
 * 
 * 
 * (c) Chris Williams, 2021.
 *
 * See LICENSE for usage and copying.
 */

use serde_derive::Deserialize;
use std::collections::HashMap;

pub const SECTIONS: [&str; 3] = [ "text", "data", "bss" ];

#[derive(Clone)]
#[derive(Deserialize)]
pub struct Config
{
    output: Output,
    section: HashMap<String, Section>
}

impl Config
{
    pub fn get_entry(&self) -> &String { &self.output.entry }
    pub fn get_sections(&self) -> &HashMap<String, Section> { &self.section }
    pub fn get_output(&self) -> &Output { &self.output }
}

#[derive(Clone)]
#[derive(Deserialize)]
pub struct Output
{
    entry: String,
    start_symbol: Option<String>,
    end_symbol: Option<String>,
    alignment: usize,
    dynamic_relocation: bool,
    base_phys_addr: Option<usize>,
    base_virt_addr: Option<usize>
}

impl Output
{
    pub fn get_alignment(&self) -> usize { self.alignment }
    pub fn get_dynamic_relocation(&self) -> bool { self.dynamic_relocation }
}

#[derive(Clone)]
#[derive(Deserialize)]
pub struct Section
{
    include: Vec<String>,
    start_symbol: Option<String>,
    end_symbol: Option<String>,
    alignment: usize
}

impl Section
{
    pub fn get_sections_to_include(&self) -> &Vec<String> { &self.include }
    pub fn get_alignment(&self) -> usize { self.alignment }
}

/* load the given file into memory and parse it, returning a config structure */
pub fn parse_config(filename: &String) -> Config
{
    let config_contents = match std::fs::read_to_string(filename)
    {
        Ok(c) => c,
        Err(e) => super::fatal_msg!("Can't read configuration file {}: {}", filename, e)
    };

    match toml::from_str(config_contents.as_str())
    {
        Ok(c) => c,
        Err(e) => super::fatal_msg!("Can't parse configutation file {}: {}", filename, e)
    }
}
