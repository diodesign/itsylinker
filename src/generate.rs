/* itsylinker ELF executable generator
 * 
 * (c) Chris Williams, 2021.
 *
 * See LICENSE for usage and copying.
 */

use std::collections::hash_map::{HashMap, Entry};
use goblin::elf::section_header::SectionHeader;
use goblin::elf::sym::Sym;

/* describe an ELF executable to generate from merging multiple ELF object files */
pub struct Executable
{
    section_headers: HashMap<String, Vec<(String, SectionHeader)>>,
    symbols: HashMap<String, (String, Sym)>
}

impl Executable
{
    pub fn new() -> Self
    {
        Self
        {
            section_headers: HashMap::new(),
            symbols: HashMap::new()
        }
    }

    /* build a big table of all section headers 
       => name = section header name
          filename = object file containing the section header
          sh = section header metadata */
    pub fn add_section_header(&mut self, name: &str, filename: &str, sh: SectionHeader)
    {
        /* hash table will map section name to filename of its object file and metadata */
        let entry = (String::from(filename), sh);

        /* some section headers will have the same names, so build a list of them
           when there is a clash and sort it out later */
        match self.section_headers.entry(String::from(name))
        {
            Entry::Vacant(v) => { v.insert(vec!(entry)); },
            Entry::Occupied(mut v) =>
            {
                v.get_mut().push(entry)
            }
        }
    }

    /* build a big table of all global symbols that need importing/resolving
       => name = symbol name
          filename = object file containing the symbol
          sym = symbol metadata
       <= true if added the symbol, false if symbol already present */
    pub fn add_global_symbol(&mut self, name: &str, filename: &str, sym: Sym) -> bool
    {
        /* hash table will map symbol to filename of its object file and metadata */
        let entry = (String::from(filename), sym);

        /* insert symbol into the table, return true if new entry */
        self.symbols.insert(String::from(name), entry).is_none()
    }
}

impl std::fmt::Debug for Executable
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        let sym_count = &self.symbols.len();

        let mut sh_count = 0;
        for (_, list) in &self.section_headers { sh_count = sh_count + list.len(); }

        write!(f, "{} section headers, {} symbols to resolve", sh_count, sym_count)?;
        Ok(())
    }
}