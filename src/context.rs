/* Centralize all context and linking here
 * 
 * The order of files on the command line is important, so store
 * the command line arguments as a stream of objects we'll step through one at a time
 * 
 * (c) Chris Williams, 2021.
 *
 * See LICENSE for usage and copying.
 */

use super::obj;
use super::rlib;
use super::section;
use super::search::Paths;
use super::config::{self, Config};
use super::generate::Executable;

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

    pub fn stream_iter(&self) -> ActionIter
    {
        ActionIter::new(&self)
    }

    /* run through the stream of actions to produce a data structure
       describing the contents of the ELF executable */
    pub fn to_executable(&self) -> Executable
    {
        /* bail out now if no config file has been loaded */
        (self.config.is_none()).then(||
            super::fatal_msg!("Linker configuration file must be specified with -T")
        );
        
        let mut paths = super::search::Paths::new();
        let mut exe = super::generate::Executable::new();

        /* bring in all section headers and the symbols */
        for item in self.stream_iter()
        {
            match item
            {
                StreamItem::SearchPath(f) => paths.add(&f),
                StreamItem::Group(g) => self.process_group(g, &mut exe, &paths),
                StreamItem::File(f) => { self.process_file(f, &mut exe, &paths); }
            }
        }

        /* arrange the layout of the sections */
        section::arrange(&self.config.clone().unwrap(), &mut exe);

        exe
    }

    /* link the given file into the given executable. return number of new unresolved references */
    fn process_file(&self, filename: String, exe: &mut Executable, paths: &Paths) -> usize
    {
        if let Some(path) = paths.find_file(&filename)
        {
            match path.as_path().extension().unwrap().to_str().unwrap()
            {
                "o" => obj::link(path, exe),
                "rlib" => rlib::link(path, exe),
                _ => super::fatal_msg!("Unrecognized file to link: {}", filename)
            }
        }
        else
        {
            super::fatal_msg!("Cannot find file {} to link", filename);
        }
    }

    /* loop through the group's files over and over until there are no new unresolved references */
    fn process_group(&self, group: Group, exe: &mut Executable, paths: &Paths)
    {
        loop
        {
            let mut new_refs = 0;

            for member in group.iter()
            {
                if let StreamItem::File(f) = member
                {
                    new_refs = new_refs + self.process_file(f.clone(), exe, paths);
                }
            }

            /* exit when we're done creating unresolved references within this group */
            if new_refs == 0
            {
                break;
            }
        }
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