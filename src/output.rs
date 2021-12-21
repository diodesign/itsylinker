/* Output an executable from collected object files
 * 
 * (c) Chris Williams, 2021.
 *
 * See LICENSE for usage and copying.
 */

use super::copy;
use super::context::Context;

use object::write::Object;
use object::write::Mangling;
use object::BinaryFormat;
use object::Architecture;
use object::endian::Endianness;

/* produce an ELF executable from the supplied configuration and command-line paramters */
pub fn write(cxt: &Context)
{
    /* bail out now if no config file has been loaded */
    let config = match cxt.get_config()
    {
        Some(config) => config,
        None => fatal_msg!("Linker configuration file must be specified with -T")
    };

    /* create a blank ELF executable that we'll output */
    let mut elf = Object::new(
        BinaryFormat::Elf,
        Architecture::Riscv64,
        Endianness::Little
    );
    elf.mangling = Mangling::None;
    elf.flags = object::FileFlags::None;

    /* produce a manifest of files to link from the config and command line settings */
    let manifest = cxt.to_manifest();

    /* copy all the required sections. this also updates the e_flags in the executable */
    let sections = copy::sections(config, &mut elf, &manifest);

    /* copy all the symbols */
    let symbols = copy::symbols(&mut elf, &manifest, &sections);

    /* copy all the relocations */
    copy::relocations(&mut elf, &manifest, &sections, &symbols);

    /* collect it all together */
    let data_to_write = match elf.write()
    {
        Ok(bytes) => bytes,
        Err(reason) => fatal_msg!("Can't generate ELF executable: {}", reason)
    };

    /* and write it all out to an executable in storage */
    if let Err(reason) = std::fs::write(&cxt.get_output_file(), data_to_write)
    {
        fatal_msg!("Unable to create executable {}: {}", cxt.get_output_file(), reason);
    }
}