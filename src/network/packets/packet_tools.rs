macro_rules! _wrap {
	($value:expr, $name:expr, $other:expr) => {
		$crate::util::error_handling::exception_wrap!($value, concat!("While reading ", $name, " packet's ", $other))?
	}
}
pub(crate) use _wrap as wrap;

macro_rules! _expect_array {
	($iterator:expr, $name:expr, $other:expr, $amount:expr) => {
		let amount = wrap!($crate::network::message_pack::reader::read_array($iterator), $name, concat!($other, " element amount"));
		if amount != $amount {
			return exception!(concat!("Wrong element amount in ", $name, " packet's ", $other, ": "), amount, concat!(" / ", $amount));
		}
	}
}
pub(crate) use _expect_array as expect_array;

macro_rules! _expect_end_of_packet {
	($iterator:expr, $packet_type:expr) => {
		if $iterator.has_more() {
			$crate::util::log_formatter::log_warn!(concat!($packet_type, " packet has more bytes than expected, "), $iterator.remaining(), " remaining bytes.");
		}
	}
}
pub(crate) use _expect_end_of_packet as expect_end_of_packet;
