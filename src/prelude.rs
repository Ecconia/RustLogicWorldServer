//All logging macros:
pub use crate::util::log_formatter::{
	log_error,
	log_warn,
	log_info,
	log_debug,
};
//Exception handling needed everywhere:
pub use crate::util::error_handling::{
	unwrap_some_or_return,
	unwrap_ok_or_return,
	EhResult,
    ExceptionDetails,
	exception,
	exception_from,
	exception_wrap,
	unwrap_or_print_return,
};
