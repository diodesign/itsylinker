/* itsylinker file finder
 * 
 * (c) Chris Williams, 2021.
 *
 * See LICENSE for usage and copying.
 */

use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct Paths
{
    paths: HashSet<String>
}

impl Paths
{
    pub fn new() -> Paths { Paths {paths: HashSet::new() } }

    pub fn add(&mut self, pathname: &String)
    {
        /* only add paths to directories */
        if Path::new(pathname).is_dir()
        {
            self.paths.insert(pathname.clone());
        }
    }

    /* get the full pathname for a file by searching for it in the current context
       and the registered search paths, or return None if it can't be found */
    pub fn find_file(&self, filename: &String) -> Option<PathBuf>
    {
        /* can we just find this file without searching? */
        if Path::new(filename).is_file()
        {
            return Some(Path::new(filename).to_path_buf());
        }

        /* if we're still here then we need to search for this file */
        for prefix in &self.paths
        {
            let mut path = Path::new(&prefix).to_path_buf();
            path.push(filename);
            if path.as_path().is_file()
            {
                return Some(path);
            }
        }

        None /* nothing found! */
    }
}