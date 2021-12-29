/* Parse the configuration file format
 * 
 * 
 * (c) Chris Williams, 2021.
 *
 * See LICENSE for usage and copying.
 */

use serde_derive::Deserialize;
use std::collections::HashMap;

#[derive(Clone)]
#[derive(Deserialize)]
pub struct Config
{
    output: Output,
    section: HashMap<String, Section>
}

impl Config
{
    pub fn get_sections(&self) -> &HashMap<String, Section> { &self.section }
    pub fn get_output(&self) -> &Output { &self.output }
}

#[derive(Clone)]
#[derive(Deserialize)]
pub struct Output
{
    entry: String,
    relocatable: bool,
    start_symbol: Option<String>,
    end_symbol: Option<String>,
    base_phys_addr: Option<u64>,
    base_virt_addr: Option<u64>
}

pub enum ExecutablePlacement
{
    Static(u64, u64), /* base phys, virt addresses */
    Relocatable
}

impl Output
{
    pub fn get_entry_symbol(&self) -> &String { &self.entry }
    pub fn get_start_symbol(&self) -> &Option<String> { &self.start_symbol }
    pub fn get_end_symbol(&self) -> &Option<String> { &self.end_symbol }
    pub fn is_relocatable(&self) -> bool { self.relocatable }

    pub fn get_placement(&self) -> ExecutablePlacement
    {
        if self.relocatable
        {
            ExecutablePlacement::Relocatable
        }
        else
        {
            /* static binaries need some sort of base load address defined.
               the ELF spec isn't terribly clear?
               if only one is given (physical or virtual) then use that for both base address fields.
               if individual phys and virt addresses are given, use those.
               use zero as base addresses if none are given rather than error. */
            match (self.base_phys_addr, self.base_virt_addr)
            {
                (None,       None)       => ExecutablePlacement::Static(0, 0),
                (None,       Some(virt)) => ExecutablePlacement::Static(virt, virt),
                (Some(phys), None)       => ExecutablePlacement::Static(phys, phys),
                (Some(phys), Some(virt)) => ExecutablePlacement::Static(phys, virt)
            }
        }
    }
}

#[derive(Clone)]
#[derive(Deserialize)]
pub struct Section
{
    include: Vec<String>,
    start_symbol: Option<String>,
    end_symbol: Option<String>
}

impl Section
{
    pub fn get_sections_to_include(&self) -> &Vec<String> { &self.include }
    pub fn get_start_symbol(&self) -> &Option<String> { &self.start_symbol }
    pub fn get_end_symbol(&self) -> &Option<String> { &self.end_symbol }
}

/* load the given file into memory and parse it, returning a config structure */
pub fn parse_config(filename: &String) -> Config
{
    let config_contents = match std::fs::read_to_string(filename)
    {
        Ok(c) => c,
        Err(e) => fatal_msg!("Can't read configuration file {}: {}", filename, e)
    };

    match toml::from_str(config_contents.as_str())
    {
        Ok(c) => c,
        Err(e) => fatal_msg!("Can't parse configutation file {}: {}", filename, e)
    }
}

/* generate a basic, default configuration. absent a configuration file, we'll
   use what's below. if a config file is specified, these defaults are discarded */
pub fn default_config() -> Config
{
    Config
    {
        /* default settings */
        output: Output
        {
            entry: String::from("_start"),
            start_symbol: None,
            end_symbol: None,
            relocatable: true,
            base_phys_addr: None,
            base_virt_addr: None
        },

        /* default sections */
        section:
        {
            let mut tbl = HashMap::new();
            for (name, section) in
            [
                ("text", Section
                {
                    include: vec![ String::from(".entry*"), String::from(".init*"), String::from(".text*") ],
                    start_symbol: None,
                    end_symbol: None
                }),
            
                ("rodata", Section
                {
                    include: vec![ String::from(".rodata*") ],
                    start_symbol: None,
                    end_symbol: None
                }),
            
                ("data", Section
                {
                    include: vec![ String::from(".data*") ],
                    start_symbol: None,
                    end_symbol: None
                }),
            
                ("bss", Section
                {
                    include: vec![ String::from(".bss*") ],
                    start_symbol: Some(String::from("__bss_start")),
                    end_symbol: Some(String::from("__bss_end"))
                })
            ]
            {
                tbl.insert(String::from(name), section);
            }
            tbl
        }
    }
}