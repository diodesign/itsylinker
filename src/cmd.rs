/* itsylinker command-line parser
 * 
 * (c) Chris Williams, 2021.
 *
 * See LICENSE for usage and copying.
 */

use super::context::{Context, Group, StreamItem};

/* use a state machine to analyze command line args */
enum State
{
    ExpectingAnything,
    ExpectingSearchPath,
    ExpectingOutputFile,
    ExpectingConfigFile,
    ExpectingFlavorType,
    WaitingForGroupEnd
}

/* convert command-line arguments into a native context structure */
pub fn parse_args() -> Context
{
    let mut context = Context::new();
    let mut state = State::ExpectingAnything;
    let mut group = Group::new();

    /* get the command-line arguments as a list of strings, skipping
    the first argument because it's just the program name */
    let arg_array = std::env::args().collect::<Vec<String>>().split_off(1);
    let args = arg_array.as_slice();

    for arg in args
    {
        match state
        {
            /* argument could be an object file or a switch. figure out
               which it is, and either change state to handle the switch
               or include the object file in the processing stream */
            State::ExpectingAnything =>
            {
                match parse_single_arg(arg)
                {
                    (true, Some(s)) => state = s,
                    (false, None) => context.add_to_stream(StreamItem::File(arg.clone())),
                    (_, _) => ()
                }
            },

            /* if we're in a group, keep adding archives to the group */
            State::WaitingForGroupEnd =>
            {
                match parse_single_arg(arg)
                {
                    (true, Some(State::ExpectingAnything)) =>
                    {
                        /* we've left the group, so commit it to the stream
                           and create a blank group for next time */
                        context.add_to_stream(StreamItem::Group(group.clone()));
                        state = State::ExpectingAnything;
                        group = Group::new();
                    },
                    (false, None) => group.add(StreamItem::File(arg.clone())),
                    (_, _) => ()
                }   
            }

            /* the argument is expected to be a search path */
            State::ExpectingSearchPath =>
            {
                context.add_to_stream(StreamItem::SearchPath(arg.clone()));
                state = State::ExpectingAnything;
            },

            /* the argument is expected to be the executable output filename */
            State::ExpectingOutputFile =>
            {
                context.set_output_file(arg);
                state = State::ExpectingAnything;
            },

            /* the argument is expected to be the linker config script filename.
               it's parsed immediately and contents stashed in the context */
            State::ExpectingConfigFile =>
            {
                context.parse_config_file(arg);
                state = State::ExpectingAnything;
            },

            State::ExpectingFlavorType =>
            {
                if arg != "gnu"
                {
                    super::fatal_msg!("{} only supports the 'gnu' interface flavor",
                        env!("CARGO_PKG_NAME"));
                }
                state = State::ExpectingAnything;
            }
        }
    }

    context
}

/* attempt to parse a single argument and return whether or not the arg
   was successfully parsed, and the new state of the parser */
fn parse_single_arg(arg: &String) -> (bool, Option<State>)
{
    /* display minimal help and exit */
    if arg == "--help"
    {
        super::fatal_msg!("Usage: {} [options] <file>...",
            env!("CARGO_BIN_NAME"));
    }

    /* display version information */
    if arg == "--version"
    {
        super::fatal_msg!("{} {} by {}",
            env!("CARGO_BIN_NAME"),
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_AUTHORS"));
    }

    /* next command line argument must be a search path */
    if arg == "-L" { return (true, Some(State::ExpectingSearchPath)) }

    /* next command line argument must be an output file name */
    if arg == "-o" { return (true, Some(State::ExpectingOutputFile)) }

    /* next command line argument must be the config filename */
    if arg == "-T" { return (true, Some(State::ExpectingConfigFile)) }

    /* next command line argument will be the interface flavor, which must be 'gnu' */
    if arg == "-flavor" { return (true, Some(State::ExpectingFlavorType)) }

    /* ignore requests to garbage collect sections: we'll do that automatically */
    if arg == "--gc-sections" { return (true, None) }

    /* ignore requests for static and dynamic: that's handled automatically from the config file */
    if arg == "-Bstatic" { return (true, None) }
    if arg == "-Bdynamic" { return (true, None) }

    /* ignore DT_NEEDED tags */
    if arg == "--as-needed" { return (true, None) }
    if arg == "--no-add-needed" { return (true, None) }

    /* stack is always non-executable */
    if arg == "-znoexecstack" { return (true, None) }

    /* put us into group mode. if we were already in group mode, continue */
    if arg == "--start-group" { return (true, Some(State::WaitingForGroupEnd)) }

    /* take us out of group, if we're in one */
    if arg == "--end-group" { return (true, Some(State::ExpectingAnything)) }

    return (false, None) /* nothing handled and no change to state */
}
