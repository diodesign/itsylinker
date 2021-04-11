/* itsylinker object file parser
 * 
 * references:
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

use super::generate::Executable;

const EM_RISCV: u16 = 243;  /* RISC-V ELF files are always this ELF machine type */

/* link the given object file into the given executable. return number of new unresolved references */
pub fn link(filename: std::path::PathBuf, exe: &mut Executable) -> usize
{
    /* load file into byte slice and process it */
    let contents = super::load_file_into_bytes(filename.clone());
    link_slice(&filename, contents.as_slice(), exe)
}

/* link the given byte slice into the given executable. return number of new unresolved references */
pub fn link_slice(source: &std::path::PathBuf, slice: &[u8], exe: &mut Executable) -> usize
{
    let mut new_symbols = 0;
    let source_filename = source.to_str().unwrap();

    /* avoid processing bad data */
    let object = match goblin::elf::Elf::parse(slice)
    {
        Ok(o) => o,
        Err(e) => super::fatal_msg!("Failed to parse object file {}: {}", source_filename, e)
    };

    /* only accept 64-bit RISC-V files */
    (object.header.e_machine != EM_RISCV).then(||
        super::fatal_msg!("Cannot parse non-RISC-V object file {} (machine type 0x{:x})",
        source_filename, object.header.e_machine));

    (object.is_64 != true).then(||
        super::fatal_msg!("Cannot parse 32-bit object file {})", source_filename));

    /* gather up section headers in this file and pair their metadata with object file's location */
    for sh in object.section_headers
    {
        if let Some(sh_name) = object.shdr_strtab.get_unsafe(sh.sh_name)
        {
            exe.add_section_header(sh_name, source_filename, sh);
        }
        else
        {
            super::fatal_msg!("Invalid section header name (index 0x{:x}) in {}", sh.sh_name, source_filename);
        }
    }

    /* add symbols that need to be imported to a global table */
    for sym in object.syms.iter()
    {
        if sym.is_import()
        {
            if let Some(sym_name) = object.strtab.get_unsafe(sym.st_name)
            {
                if exe.add_global_symbol(sym_name, source_filename, sym)
                {
                    new_symbols = new_symbols + 1;
                }
            }
            else
            {
                super::fatal_msg!("Invalid symbol name (index 0x{:x}) in {}", sym.st_name, source_filename);
            }
        }
    }

    new_symbols
}