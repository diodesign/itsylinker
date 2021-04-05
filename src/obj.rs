/* itsylinker object file parser
 * 
 * (c) Chris Williams, 2021.
 *
 * See LICENSE for usage and copying.
 */

use std::path::PathBuf;
use goblin::elf::Elf;

/* link the given object file into the final executable. return number of new unresolved references */
pub fn link(filename: PathBuf) -> usize
{
    /* load file into byte slice and process it */
    let contents = super::load_file_into_bytes(filename.clone());
    link_slice(contents.as_slice())
}

/* link the given byte slice into the final executable. return number of new unresolved references */
pub fn link_slice(slice: &[u8]) -> usize
{
    /* skip object data that's invalid -- it might be meta-data we don't care about */
    if let Ok(object) = Elf::parse(slice)
    {
        eprintln!("parsed object ({})", object.is_object_file());
    }

    0
}