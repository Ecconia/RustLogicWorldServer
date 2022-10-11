#[macro_export]
macro_rules! custom_unwrap_option_or_else {
	($val:expr, $other:tt) => {
		match $val {
			Some(x) => x,
			None => {
				$other
			}
		}
	};
}

#[macro_export]
macro_rules! custom_unwrap_result_or_else {
	($val:expr, $other:tt) => {
		match $val {
			Ok(x) => x,
			Err(message) => {
				return $other(message)
			}
		}
	};
}
