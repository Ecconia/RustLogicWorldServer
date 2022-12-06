#[macro_export]
macro_rules! _custom_unwrap_option_or_else {
	($val:expr, $other:tt) => {
		match $val {
			Some(x) => x,
			None => {
				$other
			}
		}
	};
}
pub use _custom_unwrap_option_or_else as custom_unwrap_option_or_else;

#[macro_export]
macro_rules! _custom_unwrap_result_or_else {
	($val:expr, $other:tt) => {
		match $val {
			Ok(x) => x,
			Err(message) => {
				return $other(message)
			}
		}
	};
}
pub use _custom_unwrap_result_or_else as custom_unwrap_result_or_else;

pub trait ResultErrorExt<T> {
	fn forward_error(self, while_doing_what: &str) -> Result<T, String>;
}

impl<T> ResultErrorExt<T> for Result<T, String> {
	fn forward_error(self, while_doing_what: &str) -> Result<T, String> {
		match self {
			Ok(val) => Ok(val),
			Err(err) => {
				Err(format!("{}\n -> {}", while_doing_what, err))
			}
		}
	}
}

impl<T> ResultErrorExt<T> for Result<T, std::string::FromUtf8Error> {
	fn forward_error(self, while_doing_what: &str) -> Result<T, String> {
		match self {
			Ok(val) => Ok(val),
			Err(err) => {
				Err(format!("{}\n -> {}", while_doing_what, err.to_string()))
			}
		}
	}
}

impl<T> ResultErrorExt<T> for Result<T, std::io::Error> {
	fn forward_error(self, while_doing_what: &str) -> Result<T, String> {
		match self {
			Ok(val) => Ok(val),
			Err(err) => {
				Err(format!("{}\n -> {}", while_doing_what, err.to_string()))
			}
		}
	}
}
