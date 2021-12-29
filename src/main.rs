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
 * Note: A configuration file must be provided, or defaults will be used. The config file is a toml file described in config.rs.
 * It is not compatible with other linkers.
 * 
 * References:
 * https://man7.org/linux/man-pages/man5/elf.5.html 
 * https://lwn.net/Articles/276782/
 * https://www.cs.cornell.edu/courses/cs3410/2013sp/lecture/15-linkers2-i-g.pdf
 * https://github.com/riscv/riscv-elf-psabi-doc/blob/master/riscv-elf.md
 * Linkers & Loaders, John R. Levine, https://linker.iecc.com/
 * 
 * (c) Chris Williams, 2021.
 *
 * See LICENSE for usage and copying.
 */

/* keep rust quiet about in-development code */
#![allow(dead_code)]

/* our dependencies */
extern crate toml;
extern crate serde;
extern crate serde_derive;
extern crate byterider;
extern crate wildmatch;
extern crate memmap2;
extern crate indexmap;

/* use object for reading and writing ELF assests   */
extern crate object;

#[macro_use]
mod debug;     /* provides fatal_msg() macro */
mod cmd;       /* command-line parser */
mod context;   /* describe the linking context */
mod config;    /* configuration file parser */
mod search;    /* find files for the linking process */
mod gather;    /* gather sections, symbols, and relocations */
mod output;    /* generate the ELF executable */
mod manifest;  /* manage the files to process */

/* here's the process flow of the linker:
    1. define the link context from the command line arguments and config file.
       this context identifies the work that needs to be done.
    2. iterate over the files to link, gathering their section headers,
       global symbols, and relocations. cache the file contents, too.
    3. assign sequential base addresses for the sections.
    4. write out the sections, symbols and relocations as an executable.
*/

fn main()
{
    /* figure out from command line arguments and configuration file what needs to be done.
       then write out the executable to storage */
    output::write(&cmd::parse_args());

    std::process::exit(1);
}
