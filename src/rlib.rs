/* itsylinker rlib archive parser
 * 
 * (c) Chris Williams, 2021.
 *
 * See LICENSE for usage and copying.
 */

use std::path::{PathBuf};
use goblin::archive::Archive;

/* link the given arvhive into the final executable. return number of new unresolved references */
pub fn link(filename: PathBuf) -> usize
{
    /* load archive into byte slice */
    let contents = super::load_file_into_bytes(filename.clone());
    let filename = filename.as_path().to_str().unwrap();

    let mut refs = 0;

    match Archive::parse(contents.as_slice())
    {
        Ok(arc) =>
        {
            /* extract the individual .o files from the archive and process them */
            let members = arc.members();
            for member in members
            {
                let slice = match arc.extract(member, contents.as_slice())
                {
                    Ok(s) => s,
                    Err(e) =>
                    {
                        eprintln!("Failed to extract {} from archive {}: {}", member, filename, e);
                        std::process::exit(1);
                    }
                };

                refs = refs + super::obj::link_slice(slice);
            }
        },
        Err(e) =>
        {
            eprintln!("Cannot parse archive {}: {}", filename, e);
            std::process::exit(1);
        }
    }

    refs
}