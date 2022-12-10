//All logging macros:
pub use crate::util::log_formatter::{
	log_error,
	log_warn,
	log_info,
	log_debug,
};
//Exception handling needed everywhere:
pub use crate::error_handling::{
	custom_unwrap_option_or_else,
	EhResult,
	exception,
	exception_from,
	exception_wrap,
	unwrap_or_print_return,
};
