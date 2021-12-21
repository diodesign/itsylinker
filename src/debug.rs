
/* Macros for debugging
 *
 * (c) Chris Williams, 2021.
 *
 * See LICENSE for usage and copying.
 */

/* print a message to stderr and exit immediately */
#[macro_export]
macro_rules! fatal_msg
{
    ($fmt:expr) => ({ eprintln!("{}", $fmt); std::process::exit(1); });
    ($fmt:expr, $($arg:tt)*) => ({ eprintln!($fmt, $($arg)*); std::process::exit(1); });
}