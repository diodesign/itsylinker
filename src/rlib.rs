/* itsylinker rlib archive parser
 * 
 * (c) Chris Williams, 2021.
 *
 * See LICENSE for usage and copying.
 */

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use goblin::archive::Archive;
use super::generate::Executable;

/* link the given archive file into the given executable. return number of new unresolved references */
pub fn link(filename: PathBuf, exe: &mut Executable) -> usize
{
    /* load archive file into byte slice */
    let contents = super::load_file_into_bytes(filename.clone());
    link_slice(&filename, contents.as_slice(), exe)
}

/* link the given archive slice into the given executable. return number of new unresolved references */
pub fn link_slice(source: &PathBuf, slice: &[u8], exe: &mut Executable) -> usize
{
    let mut refs = 0;

    match Archive::parse(slice)
    {
        Ok(arc) =>
        {
            /* extract the individual .o files from the archive and process them */
            let members = arc.members();
            for member in members
            {
                let slice = match arc.extract(member, slice)
                {
                    Ok(s) => s,
                    Err(e) => super::fatal_msg!("Failed to extract {} from archive {}: {}",
                                member, source.to_str().unwrap(), e)
                };

                /* only accept .o and .rlib files, skip the rest */
                match Path::new(member).extension().unwrap_or(OsStr::new("")).to_str().unwrap()
                {
                    "o" =>
                    {
                        let mut canonical_source_name = source.clone();
                        canonical_source_name.push(member);
                        refs = refs + super::obj::link_slice(&canonical_source_name, slice, exe);
                    },
                    "rlib" => refs = refs + link_slice(source, slice, exe),
                    _ => ()
                }
            }
        },
        Err(e) => super::fatal_msg!("Cannot parse archive {}: {}", source.to_str().unwrap(), e)
    }

    refs
}