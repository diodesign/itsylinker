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

/* keep rust quiet about in-development parts of the code base */
#![allow(dead_code)]

extern crate toml;
extern crate serde;
extern crate serde_derive;
extern crate byterider;
extern crate goblin;

mod cmd;     /* command-line parser */
mod context; /* describe the linking context */
mod config;  /* configuration file parser */
mod search;  /* find files for the linking process */
mod obj;     /* parse object files */
mod rlib;    /* parse rlib files */

/* print a message to stderr and exit immediately */
#[macro_export]
macro_rules! fatal_msg
{
    ($fmt:expr) => ({ eprintln!("{}", $fmt); std::process::exit(1); });
    ($fmt:expr, $($arg:tt)*) => ({ eprintln!($fmt, $($arg)*); std::process::exit(1); });
}

fn main()
{
    /* find out what needs to be done from command line arguments */
    let context = cmd::parse_args();

    /* check that we have a configuration file */
    let config_filename = match context.get_config_file()
    {
        Some(f) => f,
        None => fatal_msg!("Linker configuration file must be specified with -T")
    };

    /* find out what needs to be done from the specified configuration file */
    let _config = config::parse_config(&config_filename);

    /* get a database ready of paths to search files for in */
    let mut paths = search::Paths::new();

    /* perform the link */
    context.hit_it(&mut paths);

    std::process::exit(1);
}

/* generic function to load a file into a byte vector, or bail on error */
pub fn load_file_into_bytes(filename: std::path::PathBuf) -> Vec<u8>
{
    match std::fs::read(filename.as_path())
    {
        Ok(s) => s,
        Err(e) => fatal_msg!("Cannot raad file {} into memory: {}", filename.as_path().to_str().unwrap(), e)
    }
}