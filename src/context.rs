/* Centralize all context and linking here
 * 
 * The order of files on the command line is important, so store
 * the command line arguments as a stream of objects we'll step through one at a time
 * 
 * (c) Chris Williams, 2021.
 *
 * See LICENSE for usage and copying.
 */

use super::search::Paths;
use super::config::{ self, Config };
use super::manifest::Manifest;

pub type Filename = String;

/* we have to handle a stream of input items, which could be
   search paths or object files or archive files part of a group */
#[derive(Clone)]
pub enum StreamItem
{
    File(Filename),
    SearchPath(Filename),
    Group(Group)
}

/* handle groups of items */
#[derive(Clone)]
pub struct Group
{
    files: Vec<StreamItem>
}

impl Group
{
    pub fn new() -> Group { Group { files: Vec::new() } }
    pub fn add(&mut self, item: StreamItem) { self.files.push(item) }
    pub fn iter(&self) -> std::slice::Iter<'_, StreamItem> { self.files.iter() }
}

/* this is what we're working with: a collection of files to process */
#[derive(Clone)]
pub struct Context
{
    output_file: Filename,          /* this can be set at any time */
    input_stream: Vec<StreamItem>,  /* a list of streamed items to process */
    config: Option<Config>
}

impl Context
{
    pub fn new() -> Context
    {
        Context
        {
            /* the ld-compatible executable filename default is a.out */
            output_file: String::from("a.out"),

            /* leave the rest blank */
            config: None,
            input_stream: Vec::new(),
        }
    }

    /* retrieve the configuration in this context. panics if not defined */
    pub fn get_config(&self) -> Option<&Config> { self.config.as_ref() }

    /* functions to update and access the link context */
    pub fn add_to_stream(&mut self, item: StreamItem)
    {
        self.input_stream.push(item);
    }

    pub fn set_output_file(&mut self, path: &String)
    {
        self.output_file = path.clone();
    }

    pub fn get_output_file(&self) -> String { self.output_file.clone() }

    /* parse config file and stash contents in this context */
    pub fn parse_config_file(&mut self, path: &String)
    {
        self.config = Some(config::parse_config(&path));
    }

    fn stream_iter(&self) -> ActionIter
    {
        ActionIter::new(&self)
    }

    /* load up the given file to link into the final executable */
    fn add_file(&self, filename: &String, manifest: &mut Manifest, paths: &Paths)
    {
        if let Some(path) = paths.find_file(&filename)
        {
            manifest.add(&path);
        }
        else
        {
            fatal_msg!("Cannot find file {} to link", filename);
        }
    }

    /* load a group of files to link. a group of files is right now treated
       as a list of files to add. in future, we may need to preserve the
       grouping or act in a specific way per group */
    fn add_group(&self, group: &Group, manifest: &mut Manifest, paths: &Paths)
    {
        for member in group.iter()
        {
            if let StreamItem::File(file) = member
            {
                self.add_file(file, manifest, paths);
            }
        }
    }

    /* iterate over the stream, performing each task one by one to create
       a manifest of files to link, and return it */
    pub fn to_manifest(&self) -> Manifest
    {
        let mut paths = Paths::new();
        let mut manifest = Manifest::new();

        /* bring in all section headers and the symbols */
        for item in self.stream_iter()
        {
            match item
            {
                StreamItem::SearchPath(path) => paths.add(&path),
                StreamItem::Group(group) => self.add_group(&group, &mut manifest, &paths),
                StreamItem::File(file) => self.add_file(&file, &mut manifest, &paths)
            }   
        }

        manifest
    }
}

/* provide an iterator of actions the linker needs to perform */
pub struct ActionIter<'a>
{
    /* treat this as a stream of tasks */
    stream: std::slice::Iter<'a, StreamItem>
}

impl ActionIter<'_>
{
    pub fn new(context: &Context) -> ActionIter
    {
        ActionIter { stream: context.input_stream.iter() }
    }
}

impl Iterator for ActionIter<'_>
{
    type Item = StreamItem;

    fn next(&mut self) -> Option<StreamItem>
    {
        match self.stream.next()
        {
            Some(item) => Some(item.clone()),
            None => None
        }
    }
}