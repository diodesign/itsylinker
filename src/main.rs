/* itsylinker
 * 
 * Minimalist linker that generates 64-bit RISC-V (RV64I) ELF files
 *
 * Syntax: itsylinker [options] objects...
 * 
 * It accepts the following binutils ld-compatible command-line arguments:
 * 
 * -L <path>        Add <path> to the list of paths that will be searched for the given files to link
 * -o <output>      Generate the linked ELF executable at <output> or a.out in the current working directory if not specified
 * -T <config>      Read linker settings from configuration file <config>
 * --start-group    Mark the start of a group of files in which to resolve all possible references
 * --end-group      Mark the end of a group created by --start-group
 * 
 * --help           Display minimal usage information
 * --version        Display version information
 * 
 * Interspersed in the command line arguments are object and library files to link together to form the final ELF executable
 * 
 * (c) Chris Williams, 2021.
 *
 * See LICENSE for usage and copying.
 */

mod cmd;     /* command-line parser */
mod context; /* describe the linking context */

fn main()
{
    let context = cmd::parse_args();
    eprintln!("il: config: {} output: {}",
        context.get_config_file().unwrap_or(String::from("none")),
        context.get_output_file());

    for item in context.stream_iter()
    {
        match item
        {
            context::StreamItem::Object(f)     => eprintln!("--> object          {}", f),
            context::StreamItem::SearchPath(f) => eprintln!("--> add search path {}", f),
            context::StreamItem::Group(g) => for archive in g.iter()
            {
                match archive
                {
                    context::StreamItem::Archive(f) => eprintln!("--> group archive   {}", f),
                    _ => eprintln!("??? Unexpected item in group")
                }
            }
            _ => eprintln!("??? Unexpected item in stream")
        }
    }

    std::process::exit(1);
}