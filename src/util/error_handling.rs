//Old error handling system with custom unwrappers:

pub trait UnwrapHelperOption<A, B> {
	fn unwrap_helper(self) -> Result<A, ()>;
	fn unwrap_helper_arg(self, argument: B) -> Result<A, B>;
}

impl<A, B> UnwrapHelperOption<A, B> for Option<A> {
	fn unwrap_helper(self) -> Result<A, ()> {
		match self {
			Some(x) => Ok(x),
			None => Err(()),
		}
	}
	fn unwrap_helper_arg(self, argument: B) -> Result<A, B> {
		match self {
			Some(x) => Ok(x),
			None => Err(argument),
		}
	}
}

pub trait UnwrapHelperResult<A, B, C> {
	fn unwrap_helper(self) -> Result<A, ()>;
	fn unwrap_helper_arg<F: Fn(B) -> C>(self, closure: F) -> Result<A, C>;
}

impl<A, B, C> UnwrapHelperResult<A, B, C> for Result<A, B> {
	fn unwrap_helper(self) -> Result<A, ()> {
		match self {
			Ok(x) => Ok(x),
			Err(_) => Err(()),
		}
	}
	fn unwrap_helper_arg<F: Fn(B) -> C>(self, closure: F) -> Result<A, C> {
		match self {
			Ok(x) => Ok(x),
			Err(err) => Err(closure(err)),
		}
	}
}

#[macro_export]
macro_rules! _unwrap_or_return {
	($val:expr) => {
		match $val.unwrap_helper() {
			Ok(x) => x,
			Err(val) => return val,
		}
	};
	($val:expr, $other:expr) => {
		match $val.unwrap_helper_arg($other) {
			Ok(x) => x,
			Err(val) => return val,
		}
	};
}
pub use _unwrap_or_return as unwrap_or_return;

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
				$crate::util::log_formatter::color_meta!(),
				" @ {} | {}:{}",
				$crate::util::ansi_constants::ansi_reset!(),
			), $while, file!(), line!(), column!()));
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

// ### Newer system: #############

#[macro_export]
macro_rules! _ex {
	() => {
		(
			file!(),
			line!(),
			column!(),
			None,
		)
	};
	( $($reason:expr),* ) => {
		(
			file!(),
			line!(),
			column!(),
			Some($crate::util::log_formatter::fmt_error!( $( $reason ),* )),
		)
	};
}
pub use _ex as ex;

pub trait ExceptionWrappingResult<T, E: std::fmt::Debug> {
	fn map_ex(self, details: (&str, u32, u32, Option<String>)) -> EhResult<T>;
}

impl<T, E: std::fmt::Debug> ExceptionWrappingResult<T, E> for Result<T, E> {
	fn map_ex(self, details: (&str, u32, u32, Option<String>)) -> EhResult<T> {
		self.map_err(|e| { ExceptionDetails {
			messages: vec![format!("{:?}", e)],
		}}).wrap(details)
	}
}

pub trait ExceptionWrappingOption<T> {
	fn map_ex(self, details: (&str, u32, u32, Option<String>)) -> EhResult<T>;
}

impl<T> ExceptionWrappingOption<T> for Option<T> {
	fn map_ex(self, details: (&str, u32, u32, Option<String>)) -> EhResult<T> {
		self.ok_or_else(|| {
			if details.3.is_none() {
				ExceptionDetails {
					messages: vec!["Thing be empty".to_owned()],
				}
			} else {
				ExceptionDetails {
					messages: vec![],
				}
			}
		})
	}
}

pub trait ExceptionHandling<T> {
	fn wrap(self, c: (&str, u32, u32, Option<String>)) -> Self;
}

impl<T> ExceptionHandling<T> for EhResult<T> {
	fn wrap(self, context: (&str, u32, u32, Option<String>)) -> Self {
		self.while_doing_detailed(context.0, context.1, context.2, context.3.unwrap_or_else(|| { "/".to_owned() }))
	}
}
