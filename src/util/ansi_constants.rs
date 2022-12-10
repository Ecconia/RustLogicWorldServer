#[macro_export]
macro_rules! _ansi_reset {
	( ) => {
		"\x1b[m"
	};
}
pub use _ansi_reset as ansi_reset;

#[macro_export]
macro_rules! _ansi_rgb {
	( $r:tt, $g:tt, $b:tt) => {
		concat!("\x1b[38;2;", $r, ";", $g, ";", $b, "m")
	};
}
pub use _ansi_rgb as ansi_rgb;
