//All logging macros:
pub use crate::util::log_formatter::{
	log_error,
	log_warn,
	log_info,
	log_debug,
};
//Exception handling needed everywhere:
pub use crate::util::error_handling::{
	//Unwrap or return feature:
	unwrap_or_return,
	UnwrapHelperOption,
	UnwrapHelperResult,
	//Exception framework feature:
	EhResult,
	ExceptionWrappingResult,
	ExceptionWrappingOption,
	ExceptionHandling,
	ex,
	exception,
	unwrap_or_print_return,
};
