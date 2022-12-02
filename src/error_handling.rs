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
