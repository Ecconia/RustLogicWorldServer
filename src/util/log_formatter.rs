//### Colors: ##############

//'Debug' color, used as normal WIP printing
#[macro_export]
macro_rules! _color_debug_normal {
	() => { $crate::util::ansi_constants::ansi_rgb!(150, 150, 150) };
}
pub use _color_debug_normal as color_debug_normal;

#[macro_export]
macro_rules! _color_debug_highlight {
	() => { $crate::util::ansi_constants::ansi_rgb!(200, 200, 200) };
}
pub use _color_debug_highlight as color_debug_highlight;

//Info color, used for more important events, as heading?
#[macro_export]
macro_rules! _color_info_normal {
	() => { $crate::util::ansi_constants::ansi_rgb!(180, 255, 180) };
}
pub use _color_info_normal as color_info_normal;

#[macro_export]
macro_rules! _color_info_highlight {
	() => { $crate::util::ansi_constants::ansi_rgb!(0, 180, 0) };
}
pub use _color_info_highlight as color_info_highlight;

//Warn color, used for things that should not be, but can be ignored - probably side effects
#[macro_export]
macro_rules! _color_warn_normal {
	() => { $crate::util::ansi_constants::ansi_rgb!(255, 240, 140) };
}
pub use _color_warn_normal as color_warn_normal;

#[macro_export]
macro_rules! _color_warn_highlight {
	() => { $crate::util::ansi_constants::ansi_rgb!(255, 225, 0) };
}
pub use _color_warn_highlight as color_warn_highlight;

//Error color, used when data is lost or corrupted or further execution is impossible
#[macro_export]
macro_rules! _color_error_normal {
	() => { $crate::util::ansi_constants::ansi_rgb!(255, 100, 100) };
}
pub use _color_error_normal as color_error_normal;

#[macro_export]
macro_rules! _color_error_highlight {
	() => { $crate::util::ansi_constants::ansi_rgb!(180, 0, 0) };
}
pub use _color_error_highlight as color_error_highlight;

//Meta color, used in stacktraces for the arrow and the path:
#[macro_export]
macro_rules! _color_meta {
	() => { $crate::util::ansi_constants::ansi_rgb!(180, 255, 180) };
}
pub use _color_meta as color_meta;

//### Printers: ############

#[macro_export]
macro_rules! _log_debug {
	( $( $rest:expr ),* ) => {
		println!("{}", $crate::util::log_formatter::fmt_debug!($( $rest ),*))
	};
}
pub use _log_debug as log_debug;

#[macro_export]
macro_rules! _log_info {
	( $( $rest:expr ),* ) => {
		println!("{}", $crate::util::log_formatter::fmt_info!($( $rest ),*))
	};
}
pub use _log_info as log_info;

#[macro_export]
macro_rules! _log_warn {
	( $( $rest:expr ),* ) => {
		println!("{}", $crate::util::log_formatter::fmt_warn!($( $rest ),*))
	};
}
pub use _log_warn as log_warn;

#[macro_export]
macro_rules! _log_error {
	( $( $rest:expr ),* ) => {
		println!("{}", $crate::util::log_formatter::fmt_error!($( $rest ),*))
	};
}
pub use _log_error as log_error;

//### Log-Formatters: #############

#[macro_export]
macro_rules! _fmt_debug {
	( $( $rest:expr ),* ) => {
		$crate::util::log_formatter::format_generic!{
			{
				$crate::util::log_formatter::color_debug_normal!(),
				$crate::util::log_formatter::color_debug_highlight!()
			} $( $rest ),*
		}
	}
}
pub use _fmt_debug as fmt_debug;

#[macro_export]
macro_rules! _fmt_info {
	( $( $rest:expr ),* ) => {
		$crate::util::log_formatter::format_generic!{
			{
				$crate::util::log_formatter::color_info_normal!(),
				$crate::util::log_formatter::color_info_highlight!()
			} $( $rest ),*
		}
	}
}
pub use _fmt_info as fmt_info;

#[macro_export]
macro_rules! _fmt_warn {
	( $( $rest:expr ),* ) => {
		$crate::util::log_formatter::format_generic!{
			{
				$crate::util::log_formatter::color_warn_normal!(),
				$crate::util::log_formatter::color_warn_highlight!()
			} $( $rest ),*
		}
	}
}
pub use _fmt_warn as fmt_warn;

#[macro_export]
macro_rules! _fmt_error {
	( $( $rest:expr ),* ) => {
		$crate::util::log_formatter::format_generic!{
			{
				$crate::util::log_formatter::color_error_normal!(),
				$crate::util::log_formatter::color_error_highlight!()
			} $( $rest ),*
		}
	}
}
pub use _fmt_error as fmt_error;

//### Generic Formatter: ##########

#[macro_export]
macro_rules! _format_generic {
	( {$fg:expr, $hl:expr} $a:tt ) => {
		String::from(concat!(
			$fg, $a, $crate::util::ansi_constants::ansi_reset!()
		))
	};
	( {$fg:expr, $hl:expr} $a:tt, $b:tt ) => {
		format!(
			concat!(
				$fg, $a, $hl, "{}", $crate::util::ansi_constants::ansi_reset!()
			),
			$b
		)
	};
	( {$fg:expr, $hl:expr} $( $a:tt, $b:tt ),* ) => {
		format!(
			concat!(
				$($fg, $a, $hl, "{}"),*,
				$crate::util::ansi_constants::ansi_reset!()
			),
			$($b),*
		)
	};
	( {$fg:expr, $hl:expr} $lit:tt, $( $a:tt, $b:tt ),* ) => {
		format!(
			concat!(
				$fg, $lit, $($hl, "{}", $fg, $b),*,
				$crate::util::ansi_constants::ansi_reset!()
			),
			$($a ),*
		)
	};
}
pub use _format_generic as format_generic;
