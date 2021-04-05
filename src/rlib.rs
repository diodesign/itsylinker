/* itsylinker rlib archive parser
 * 
 * (c) Chris Williams, 2021.
 *
 * See LICENSE for usage and copying.
 */

use std::path::{PathBuf};

/* link the given arvhive into the final executable. return number of new unresolved references */
pub fn link(filename: PathBuf) -> usize
{
    eprintln!("Linking archive file {}", filename.as_path().to_str().unwrap());
    0
}