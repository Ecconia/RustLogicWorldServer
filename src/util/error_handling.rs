// ### Unwrap & Return feature:

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

// ### Exception framework:

pub type EhResult<T> = Result<T, Stacktrace>;

type MetaInformation = (&'static str, u32, u32); //File-Name, Line-Number, Colon-Number

#[macro_export]
macro_rules! _meta {
	() => {
		(file!(), line!(), column!())
	};
}
pub use _meta as meta;

#[derive(Debug)]
pub struct StackFrame {
	pub file_location: &'static str,
	pub file_line: u32,
	pub file_column: u32,
	pub message: Option<String>,
}

#[derive(Debug)]
pub struct Stacktrace {
	pub message: Option<String>,
	pub frames: Vec<StackFrame>,
}

impl Stacktrace {
	//TODO: Format messages when printing it and allowing different color schemes while doing so (-> print as warn or error)
	pub fn print(&self) {
		let mut frames = &self.frames[..];
		let mut message = match &self.message {
			Some(message) => {
				format!(concat!(
					crate::util::log_formatter::color_error_normal!(), "Error: {}",
					crate::util::ansi_constants::ansi_reset!(),
				), message)
			}
			None => {
				let frame = &frames[0];
				frames = &frames[1..];
				format!(
					concat!(
						crate::util::log_formatter::color_meta!(), "Error: ",
						crate::util::log_formatter::color_error_normal!(), "{}",
						crate::util::log_formatter::color_meta!(), " @ {} | {}:{}",
						crate::util::ansi_constants::ansi_reset!(),
					),
					if frame.message.is_some() { frame.message.as_ref().unwrap().to_owned() } else { "---".to_owned() },
					frame.file_location, frame.file_line, frame.file_column
				)
			}
		};
		for frame in frames {
			message.push_str(&format!(
				concat!(
					"\n",
					crate::util::log_formatter::color_meta!(), "-> ",
					crate::util::log_formatter::color_error_normal!(), "{}",
					crate::util::log_formatter::color_meta!(), " @ {} | {}:{}",
					crate::util::ansi_constants::ansi_reset!(),
				),
				if frame.message.is_some() { frame.message.as_ref().unwrap().to_owned() } else { "---".to_owned() },
				frame.file_location, frame.file_line, frame.file_column
			));
		}
		println!("{}", message);
	}
}

#[macro_export]
macro_rules! _exception {
	( $while:expr ) => {
		Err($crate::util::error_handling::Stacktrace {
			message: None,
			frames: vec![$crate::util::error_handling::StackFrame {
				file_location: file!(),
				file_line: line!(),
				file_column: column!(),
				message: Some($while.to_owned()),
			}],
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
	fn while_doing(self, meta: MetaInformation, message: Option<String>) -> EhResult<T>;
}

impl<T> ResultExceptionDetailsExt<T> for EhResult<T> {
	fn while_doing(self, meta: MetaInformation, message: Option<String>) -> EhResult<T> {
		match self {
			Ok(val) => Ok(val),
			Err(mut err) => {
				err.frames.push(StackFrame {
					file_location: meta.0,
					file_line: meta.1,
					file_column: meta.2,
					message,
				});
				Err(err)
			}
		}
	}
}

// ### Newer system: #############

//TODO: Move evaluation of errors to the printing stage, by storing closures instead.
//TODO: Move formatting to runtime, or wrap with custom symbols to replace them later on.

//Creation:

#[macro_export]
macro_rules! _ex {
	() => {
		($crate::util::error_handling::meta!(), None)
	};
	( $($reason:expr),* ) => {
		($crate::util::error_handling::meta!(), Some($crate::util::log_formatter::fmt_error!( $( $reason ),* )))
	};
}
pub use _ex as ex;

pub trait ExceptionWrappingResult<T, E: std::fmt::Debug> {
	fn map_ex(self, c: (MetaInformation, Option<String>)) -> EhResult<T>;
}

impl<T, E: std::fmt::Debug> ExceptionWrappingResult<T, E> for Result<T, E> {
	fn map_ex(self, context: (MetaInformation, Option<String>)) -> EhResult<T> {
		self.map_err(|e| {
			let meta = context.0;
			let message = context.1;
			let error = format!("{:?}", e);
			Stacktrace {
				message: Some(error),
				frames: vec![StackFrame {
					file_location: meta.0,
					file_line: meta.1,
					file_column: meta.2,
					message,
				}],
			}
		})
	}
}

pub trait ExceptionWrappingOption<T> {
	fn map_ex(self, c: (MetaInformation, Option<String>)) -> EhResult<T>;
}

impl<T> ExceptionWrappingOption<T> for Option<T> {
	fn map_ex(self, context: (MetaInformation, Option<String>)) -> EhResult<T> {
		self.ok_or_else(|| {
			let meta = context.0;
			let message = context.1;
			Stacktrace {
				message: None,
				frames: vec![StackFrame {
					file_location: meta.0,
					file_line: meta.1,
					file_column: meta.2,
					message,
				}],
			}
		})
	}
}

//Wrapping:

pub trait ExceptionHandling<T> {
	fn wrap(self, c: (MetaInformation, Option<String>)) -> Self;
}

impl<T> ExceptionHandling<T> for EhResult<T> {
	fn wrap(self, context: (MetaInformation, Option<String>)) -> Self {
		self.while_doing(context.0, context.1)
	}
}

// ### Unwrap & Print:

#[macro_export]
macro_rules! _unwrap_or_print_return {
	($val:expr) => {
		match $val {
			Ok(x) => x,
			Err(message) => {
				$crate::util::error_handling::Stacktrace::print(&message);
				return;
			}
		}
	};
}
pub use _unwrap_or_print_return as unwrap_or_print_return;
