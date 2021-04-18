/* itsylinker object file parser
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

    /* cache this object file's contents so it can be used later to build the final executable */
    exe.cache_section_source(source_filename, slice);

    /* gather up section headers in this file and store their contents and metadata in a big table */
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