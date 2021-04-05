/* itsylinker
 * 
 * Minimalist linker that generates 64-bit RISC-V (RV64I) ELF files
 *
 * Syntax: itsylinker [options] objects...
 * 
 * It accepts the following binutils ld-compatible command-line arguments:
 * 
 * -L <path>        Add <path> to the list of paths that will be searched for the given files to link
 * -o <output>      Generate the linked ELF executable at <output> or a.out in the current working directory if not specified
 * -T <config>      Read linker settings from configuration file <config>
 * --start-group    Mark the start of a group of files in which to resolve all possible references
 * --end-group      Mark the end of a group created by --start-group
 * 
 * --help           Display minimal usage information
 * --version        Display version information
 * 
 * Interspersed in the command line arguments are object and library files to link together to form the final ELF executable.
 * Note: A configuration file must be provided. This is a toml file described in config.rs. It is not compatible with other linkers.
 * 
 * (c) Chris Williams, 2021.
 *
 * See LICENSE for usage and copying.
 */

extern crate toml;
extern crate serde;
extern crate serde_derive;
extern crate byterider;
extern crate goblin;

use std::path::PathBuf;

mod cmd;     /* command-line parser */
mod context; /* describe the linking context */
mod config;  /* configuration file parser */
mod search;  /* find files for the linking process */
mod obj;     /* parse object files */
mod rlib;    /* parse rlib files */

fn main()
{
    /* find out what needs to be done from command line arguments */
    let context = cmd::parse_args();
    let config_filename = match context.get_config_file()
    {
        Some(f) => f,
        None =>
        {
            eprintln!("Linker configuration file must be specified with -T");
            std::process::exit(1);
        }
    };

    /* find out what needs to be done from the specified configuration file */
    let config = config::parse_config(&config_filename);
    eprintln!("il: entry symbol = {}", config.get_entry());

    /* get a database ready of paths to search files for in */
    let mut paths = search::Paths::new();
    
    /* run through a stream of actions to take to complete the linking process */
    for item in context.stream_iter()
    {
        match item
        {
            context::StreamItem::SearchPath(f) => paths.add(&f),
            context::StreamItem::Group(g) => process_group(g, &paths),
            context::StreamItem::File(f) => { process_file(f, &paths); }
        }
    }

    std::process::exit(1);
}

/* link the given file into the final executable. return number of new unresolved references */
fn process_file(filename: String, paths: &search::Paths) -> usize
{
    if let Some(path) = paths.find_file(&filename)
    {
        /* don't pass bad filenames to the linker, it will bail out in a panic */
        match path.as_path().extension().unwrap().to_str().unwrap()
        {
            "o" => obj::link(path),
            "rlib" => rlib::link(path),
            ext =>
            {
                eprintln!("Unrecognized file to link: {}", ext);
                std::process::exit(1);
            }
        }
    }
    else
    {
        eprintln!("Cannot find file {} to link", filename);
        std::process::exit(1);
    }
}

/* loop through the group's files over and over until there are no new unresolved references */
fn process_group(group: context::Group, paths: &search::Paths)
{
    loop
    {
        let mut refs = 0;

        for member in group.iter()
        {
            match member
            {
                context::StreamItem::File(f) => refs = refs + process_file(f.clone(), paths),
                _ => () /* ignore non-files */
            }
        }

        /* exit when we're done creating unresolved references within this group */
        if refs == 0
        {
            break;
        }
    }
}

/* generic function to load a file into a byte vector, or bail on error */
pub fn load_file_into_bytes(filename: PathBuf) -> Vec<u8>
{
    match std::fs::read(filename.as_path())
    {
        Ok(s) => s,
        Err(e) =>
        {
            eprintln!("Cannot load object file {}: {}", filename.as_path().to_str().unwrap(), e);
            std::process::exit(1);
        }
    }
}