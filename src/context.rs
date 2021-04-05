/* Centralize all the context we can about a particular linking task
 * 
 * The order of files on the command line is important, so store
 * the command line arguments as a stream of objects we'll step through one at a time
 * 
 * (c) Chris Williams, 2021.
 *
 * See LICENSE for usage and copying.
 */

pub type Filename = String;

/* we have to handle a stream of input items, which could be
   search paths or object files or archive files part of a group */
#[derive(Clone)]
pub enum StreamItem
{
    Object(Filename),
    Archive(Filename),
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
    config_file: Option<Filename>,  /* this can be set at any time */
    input_stream: Vec<StreamItem>,         /* a list of streamed items to process */
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
            config_file: None,
            input_stream: Vec::new(),
        }
    }

    pub fn add_to_stream(&mut self, item: StreamItem)
    {
        self.input_stream.push(item);
    }

    pub fn set_output_file(&mut self, path: &String)
    {
        self.output_file = path.clone();
    }

    pub fn set_config_file(&mut self, path: &String)
    {
        self.config_file = Some(path.clone());
    }

    pub fn get_output_file(&self) -> String { self.output_file.clone() }
    pub fn get_config_file(&self) -> Option<String> { self.config_file.clone() }

    pub fn stream_iter(&self) -> ActionIter
    {
        ActionIter::new(&self)
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