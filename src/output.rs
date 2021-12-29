/* Output an executable from collection of object files
 * 
 * use gather.rs to produce a rough draft of
 * the executable and then organize everything needed
 * into a final executable
 * 
 * (c) Chris Williams, 2021.
 *
 * See LICENSE for usage and copying.
 */

use super::gather;
use super::context::Context;

use object::endian::Endianness;
use object::write::elf::Writer;

/* produce an ELF executable from the supplied configuration and command-line paramters */
pub fn write(cxt: &Context)
{
    let config = cxt.get_config();

    /* produce a manifest of files to link from the config and command line settings */
    let manifest = cxt.to_manifest();

    /* collect and arrange all the required sections. this also updates the e_flags in the executable */
    let mut sections = gather::Collection::new(config, &manifest);
    sections.merge();
    sections.arrange(&manifest);

    /* start generating the executable */
    let mut output_buffer = Vec::new();
    let mut writer = Writer::new(Endianness::Little, true, &mut output_buffer);

    writer.reserve_file_header();

    /* and write it all out to an executable in storage */
    if let Err(reason) = std::fs::write(&cxt.get_output_file(), output_buffer)
    {
        fatal_msg!("Unable to create executable file {}: {}", cxt.get_output_file(), reason);
    }
}