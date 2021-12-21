/* Manifest of files for itsylinker to link
 * 
 * (c) Chris Williams, 2021.
 *
 * See LICENSE for usage and copying.
 */

use std::fs::File;
use object::Object;
use std::path::{ Path, PathBuf };
use memmap2::{ MmapOptions, Mmap };
use std::collections::HashMap;

pub type FileIdentifier = PathBuf;

/* a manifest is a map of file identifiers to their placement in memory */
pub struct Manifest
{
    data: HashMap<FileIdentifier, Mmap>
}

/* manage the manifest of files */
impl Manifest
{
    /* create a new empty manifest */
    pub fn new() -> Manifest
    {
        Manifest
        {
            data: HashMap::new()
        }
    }

    /* map a file to memory and add it to the manifest.
       this is the outward-facing interface to the manifest structure */
    pub fn add(&mut self, filename: &PathBuf)
    {
        let mapping = self.map_file(filename, None, None);

        /* a note about filename versus psuedo-path:
           the filename is the canonical, real location of the data in storage.
           the psuedo-path treats .rlib files as if they were directories of files.
           the psuedo-path therefore serves two purposes: allowing files to be identified from
           their extensions (.o, .rlib), and for uniquely ID'ing individual files.

           if an object file called bar.o is in an archive called foo.rlib then:
           filename = foo.rlib (the source of the data)
           psuedo_path = foo.rlib/bar.o (rlib turned into a psuedo directory)
           
           if an object file is called bar.o and is not in any rlib, then:
           filename = bar.o (the source of the data)
           psuedo_path = bar.o */
        let psuedo_path = filename.clone();
        self.add_file(filename, &psuedo_path, mapping);
    }

    /* internal front-end to add_object() and expand_archive(). 
       add the given memory-mapped file. use the psuedo-path to detect the file-type
       => filename = source of the memory-mapped file in storage
          psuedo_path = identifier for the file based on its filename
          mapping = where in memory the file is stored */
    fn add_file(&mut self, filename: &PathBuf, psuedo_path: &FileIdentifier, mapping: Mmap)
    {
        match psuedo_path.as_path().extension().unwrap().to_str().unwrap()
        {
            "o" => self.add_object(psuedo_path, mapping),
            "rlib" => self.expand_archive(filename, psuedo_path, mapping),
            "rmeta" => (), /* skip metadata */
            _ => fatal_msg!("Unrecognized file to link: {}", psuedo_path.to_str().unwrap())
        };
    }
    
    /* validate an object file in memory and add it to the manifest if all OK */
    fn add_object(&mut self, psuedo_path: &FileIdentifier, mapping: Mmap)
    {
        /* avoid processing bad or unsupported data */
        let object = parse(&mapping);
        (object.format() != object::BinaryFormat::Elf).then(||
            fatal_msg!("Unsupported binary format {}: {:?}", psuedo_path.to_str().unwrap(), object.format())
        );

        /* only accept 64-bit RISC-V object files */
        (object.architecture() != object::Architecture::Riscv64).then(||
            fatal_msg!("Can't parse non-RISC-V object file {}, type {:?}",
            psuedo_path.to_str().unwrap(), object.architecture()));

        self.data.insert(psuedo_path.clone(), mapping);
    }

    /* iterate over an archive mapped into memory */
    fn expand_archive(&mut self, filename: &PathBuf, psuedo_path: &FileIdentifier, mapping: Mmap)
    {
        let archive = match object::read::archive::ArchiveFile::parse(&*mapping)
        {
            Ok(parsed) => parsed,
            Err(reason) => fatal_msg!("Can't parse archive file {}: {}", psuedo_path.to_str().unwrap(), reason)
        };

        for member in archive.members()
        {
            match member
            {
                Ok(member) =>
                {
                    /* map this archive member into memory */
                    let (offset, length) = member.file_range();
                    let sub_mapping = self.map_file(&filename, Some(offset), Some(length as usize));

                    /* append the member name to the psuedo-path so the member can be identified next
                       time around from its extension, eg: .o or .rlib, and also uniquely identified
                       if within an rlib */
                    let mut next_psuedo_path = psuedo_path.clone();
                    next_psuedo_path.push(Path::new(std::str::from_utf8(member.name()).unwrap()));
                    self.add_file(&filename, &next_psuedo_path, sub_mapping);
                },
                Err(reason) => fatal_msg!("Can't parse contents of archive file {}: {}", psuedo_path.to_str().unwrap(), reason)
            }
        }
    }

    /* map a file into memory, from offset within the file, and length number of bytes.
       if offset and/or length are not specified then use the defaults (offset = 0, size = length of file) */
    fn map_file(&self, filename: &PathBuf, offset: Option<u64>, length: Option<usize>) -> Mmap
    {
        let file = match File::open(filename)
        {
            Ok(file) => file,
            Err(reason) => fatal_msg!("Can't open file {}: {}", filename.to_str().unwrap(), reason)
        };

        let mut mapping = MmapOptions::new();
        
        if let Some(offset) = offset
        {
            mapping.offset(offset);
        }

        if let Some(length) = length
        {
            mapping.len(length);
        }

        /* if the files mapped into itsylinker change during the linking process then we'll probably crash */
        match unsafe { mapping.map(&file) }
        {
            Ok(mmap) => mmap,
            Err(reason) => fatal_msg!("Can't map file {} to memory: {}", filename.to_str().unwrap(), reason)
        }
    }

    /* iterate over all the memory-mapped object files in the manifest */
    pub fn raw_objects(&self) -> std::collections::hash_map::Iter<PathBuf, Mmap>
    {
        self.data.iter()
    }
}

/* parse raw memory-mapped data into an object */
pub fn parse(mmap: &Mmap) -> object::File
{
    match object::File::parse(&**mmap)
    {
        Ok(parsed) => parsed,
        Err(reason) => fatal_msg!("Couldn't parse mmeory-mapped object {:?}: {}", mmap, reason)
    }
}