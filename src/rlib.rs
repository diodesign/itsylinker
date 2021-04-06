/* itsylinker rlib archive parser
 * 
 * (c) Chris Williams, 2021.
 *
 * See LICENSE for usage and copying.
 */

/* link the given arvhive into the final executable. return number of new unresolved references */
pub fn link(filename: std::path::PathBuf) -> usize
{
    /* load archive into byte slice */
    let contents = super::load_file_into_bytes(filename.clone());
    let filename = filename.as_path().to_str().unwrap();

    let mut refs = 0;

    match goblin::archive::Archive::parse(contents.as_slice())
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
                    Err(e) => super::fatal_msg!("Failed to extract {} from archive {}: {}", member, filename, e)
                };

                refs = refs + super::obj::link_slice(slice);
            }
        },
        Err(e) => super::fatal_msg!("Cannot parse archive {}: {}", filename, e)
    }

    refs
}