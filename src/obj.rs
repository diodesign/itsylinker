/* itsylinker object file parser
 * 
 * (c) Chris Williams, 2021.
 *
 * See LICENSE for usage and copying.
 */

/* link the given object file into the final executable. return number of new unresolved references */
pub fn link(filename: std::path::PathBuf) -> usize
{
    /* load file into byte slice and process it */
    let contents = super::load_file_into_bytes(filename.clone());
    link_slice(contents.as_slice())
}

/* link the given byte slice into the final executable. return number of new unresolved references */
pub fn link_slice(slice: &[u8]) -> usize
{
    /* skip object data that's invalid -- it might be meta-data we don't care about */
    let object = match goblin::elf::Elf::parse(slice)
    {
        Ok(o) => o,
        Err(_) => return 0
    };

    match object.strtab.to_vec()
    {
        Ok(v) => for symbol in v
        {
            eprintln!("found symbol {}", symbol)
        },
        Err(e) => eprintln!("Can't read symbol table: {}", e)
    }

    0
}