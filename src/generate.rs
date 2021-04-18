/* itsylinker ELF executable generator
 * 
 * (c) Chris Williams, 2021.
 *
 * See LICENSE for usage and copying.
 */

use std::collections::hash_map::{HashMap, Entry, Iter};
use goblin::elf::section_header::SectionHeader;
use goblin::elf::sym::Sym;

/* describe where a section header will be placed in memory */
pub struct SectionLayout
{
    mem_offset: usize,
    filename: String,
    sh: SectionHeader
}

impl SectionLayout
{
    pub fn new(mem_offset: usize, filename: String, sh: SectionHeader) -> SectionLayout
    {
        SectionLayout
        {
            mem_offset, filename, sh
        }
    }
}

pub struct SectionSource
{
    name: String,
    filename: String,
    sh: SectionHeader
}

impl SectionSource
{
    pub fn get_filename(&self) -> &String { &self.filename }
    pub fn get_name(&self) -> &String { &self.name }
    pub fn get_sh(&self) -> &SectionHeader { &self.sh }
}

/* describe an ELF executable to generate from merging multiple ELF object files */
pub struct Executable
{
    section_headers: HashMap<String, Vec<SectionSource>>,
    symbols: HashMap<String, (String, Sym)>,
    layout: Vec<SectionLayout>,
    cache: HashMap<String, Vec<u8>>
}

impl Executable
{
    pub fn new() -> Self
    {
        Self
        {
            section_headers: HashMap::new(),
            symbols: HashMap::new(),
            layout: Vec::new(),
            cache: HashMap::new()
        }
    }

    /* retain a copy of the given binary blob in a table
       that's indexed by the blob's unique identifier.
       this is where we keep all our individual .o files */
    pub fn cache_section_source(&mut self, identifier: &str, slice: &[u8])
    {
        if self.cache.insert(String::from(identifier), Vec::from(slice)) != None
        {
            eprintln!("Warning: overwriting {} in section header cache", identifier);
        }
    }

    /* build a big table of all section headers 
       => name = section header name
          filename = object file containing the section header
          sh = section header metadata */
    pub fn add_section_header(&mut self, name: &str, filename: &str, sh: SectionHeader)
    {
        /* hash table will map section name to filename of its object file and metadata */
        let entry = SectionSource
        {
            filename: String::from(filename),
            name: String::from(name),
            sh
        };

        /* some section headers will have the same names, so build a list of them
           when there is a clash and sort it out later */
        match self.section_headers.entry(String::from(name))
        {
            Entry::Vacant(v) =>
            {
                v.insert(vec!(entry));
            },
            Entry::Occupied(mut v) =>
            {
                v.get_mut().push(entry)
            }
        }
    }

    pub fn iter_section_headers(&self) -> Iter<'_, String, Vec<SectionSource>>
    {
        self.section_headers.iter()
    }

    /* build a table of locations in memory where section headers should go
       => offset = offset from start of virtual or physical memory where
                   section header's content should start
          filename = object file containing the section header
          sh = section header metadata */
    pub fn add_to_layout(&mut self, layout: SectionLayout)
    {
        /* check we've cached that file */
        if self.cache.contains_key(&layout.filename) == false
        {
            super::fatal_msg!("Can't find cache entry for {}", layout.filename);
        }

        eprintln!("{:x}-{:x} from {}", layout.mem_offset, layout.mem_offset + layout.sh.sh_size as usize, layout.filename);
        self.layout.push(layout);
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