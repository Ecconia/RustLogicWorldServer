//Old error handling system with custom unwrappers:

#[macro_export]
macro_rules! _unwrap_some_or_return {
	($val:expr) => {
		match $val {
			Some(x) => x,
			None => return,
		}
	};
	($val:expr, $other:expr) => {
		match $val {
			Some(x) => x,
			None => return $other,
		}
	};
}
pub use _unwrap_some_or_return as unwrap_some_or_return;

#[macro_export]
macro_rules! _unwrap_ok_or_return {
	($val:expr) => {
		match $val {
			Ok(x) => x,
			Err(err) => return,
		}
	};
	($val:expr, $other:expr) => {
		match $val {
			Ok(x) => x,
			Err(error) => {
				return $other(error)
			}
		}
	};
}
pub use _unwrap_ok_or_return as unwrap_ok_or_return;

// ### New system: #############

pub type EhResult<T> = Result<T, ExceptionDetails>;

#[derive(Debug)]
pub struct ExceptionDetails {
	pub messages: Vec<String>,
}

impl ExceptionDetails {
	//TODO: Format messages when printing it and allowing different color schemes while doing so (-> print as warn or error)
	pub fn print(&self) {
		println!("{}", self.messages.join("\n"));
	}
}

#[macro_export]
macro_rules! _unwrap_or_print_return {
	($val:expr) => {
		match $val {
			Ok(x) => x,
			Err(message) => {
				$crate::util::error_handling::ExceptionDetails::print(&message);
				return;
			}
		}
	};
}
pub use _unwrap_or_print_return as unwrap_or_print_return;

#[macro_export]
macro_rules! _exception {
	( $while:expr ) => {
		Err({
			let mut messages = Vec::new();
			messages.push(format!(concat!(
				$crate::util::log_formatter::color_error_normal!(),
				"Error: {}",
				$crate::util::ansi_constants::ansi_reset!(),
			), $while));
			$crate::util::error_handling::ExceptionDetails {
				messages
			}
		})
	};
	( $( $while:expr ),+ ) => {
		$crate::util::error_handling::exception!(
			$crate::util::log_formatter::fmt_error!($($while),+)
		)
	};
}
pub use _exception as exception;

#[macro_export]
macro_rules! _exception_from {
	( $result:expr ) => {
		//No explanation of when this exception was thrown, just wrap it...
		// Not recommended, as it won't add any position details.
		match $result {
			Ok(value) => Ok(value),
			Err(exception) => $crate::util::error_handling::exception!(format!("{:?}", exception))
		}
	};
	( $result:expr, $( $while:expr ),+ ) => {
		//Exception while... basically syntax sugar for expansion...
		$crate::util::error_handling::exception_wrap!(
			$crate::util::error_handling::exception_from!($result),
			$( $while ),+
		)
	};
}
pub use _exception_from as exception_from;

pub trait ResultExceptionDetailsExt<T> {
	fn while_doing(self, while_doing_what: &str) -> EhResult<T>;
	fn while_doing_detailed(self, file_location: &str, file_line: u32, file_column: u32, while_doing_what: String) -> EhResult<T>;
}

impl<T> ResultExceptionDetailsExt<T> for EhResult<T> {
	fn while_doing(self, while_doing_what: &str) -> EhResult<T> {
		match self {
			Ok(val) => Ok(val),
			Err(mut err) => {
				err.messages.push(format!(concat!(
					crate::util::log_formatter::color_meta!(),
					" -> ",
					crate::util::log_formatter::color_error_normal!(),
					"{}",
					crate::util::ansi_constants::ansi_reset!(),
				), while_doing_what));
				err.messages.push(while_doing_what.to_string());
				Err(err)
			}
		}
	}
	
	fn while_doing_detailed(self, file_location: &str, file_line: u32, file_column: u32, while_doing_what: String) -> EhResult<T> {
		match self {
			Ok(val) => Ok(val),
			Err(mut err) => {
				err.messages.push(format!(concat!(
					crate::util::log_formatter::color_meta!(),
					" -> {}",
					crate::util::log_formatter::color_meta!(),
					" @ {} | {}:{}",
					crate::util::ansi_constants::ansi_reset!(),
				), while_doing_what, file_location, file_line, file_column));
				Err(err)
			}
		}
	}
}

#[macro_export]
macro_rules! _exception_wrap {
	( $result:expr, $( $while:expr ),* ) => {
		$crate::util::error_handling::ResultExceptionDetailsExt::while_doing_detailed(
			$result, file!(), line!(), column!(), $crate::util::log_formatter::fmt_error!($( $while ),*)
		)
	};
}
pub use _exception_wrap as exception_wrap;
