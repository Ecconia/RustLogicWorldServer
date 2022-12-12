use crate::prelude::*;

use crate::util::custom_iterator::CustomIterator;

//Integers:

pub fn read_int_8(iterator: &mut CustomIterator) -> EhResult<u8> {
	exception_wrap!(iterator.next(), "While reading a MP 8 bit integer")
}

pub fn read_int_16(iterator: &mut CustomIterator) -> EhResult<u16> {
	if iterator.remaining() < 2 {
		return exception!("While reading a MP 16 bit integer, ran out of bytes: ", iterator.remaining(), "/", 2);
	}
	Ok((iterator.next_unchecked() as u16) << 8 | iterator.next_unchecked() as u16)
}

pub fn read_int_32(iterator: &mut CustomIterator) -> EhResult<u32> {
	if iterator.remaining() < 4 {
		return exception!("While reading a MP 32 bit integer, ran out of bytes: ", iterator.remaining(), "/", 4);
	}
	Ok((iterator.next_unchecked() as u32) << 24 |
		(iterator.next_unchecked() as u32) << 16 |
		(iterator.next_unchecked() as u32) << 8 |
		(iterator.next_unchecked() as u32))
}

pub fn read_int_64(iterator: &mut CustomIterator) -> EhResult<u64> {
	if iterator.remaining() < 8 {
		return exception!("While reading MP 64 bit integer, ran out of bytes: ", iterator.remaining(), "/", 4);
	}
	Ok((iterator.next_unchecked() as u64) << 56 |
		(iterator.next_unchecked() as u64) << 48 |
		(iterator.next_unchecked() as u64) << 40 |
		(iterator.next_unchecked() as u64) << 32 |
		(iterator.next_unchecked() as u64) << 24 |
		(iterator.next_unchecked() as u64) << 16 |
		(iterator.next_unchecked() as u64) << 8 |
		(iterator.next_unchecked() as u64))
}

pub fn read_sint_8(iterator: &mut CustomIterator) -> EhResult<i8> {
	match read_int_8(iterator) {
		Ok(val) => Ok(val as i8),
		Err(e) => exception_wrap!(Err(e), "While reading signed 8 bit MP integer")
	}
}

pub fn read_sint_16(iterator: &mut CustomIterator) -> EhResult<i16> {
	match read_int_16(iterator) {
		Ok(val) => Ok(val as i16),
		Err(e) => exception_wrap!(Err(e), "While reading signed 16 bit MP integer")
	}
}

pub fn read_sint_32(iterator: &mut CustomIterator) -> EhResult<i32> {
	match read_int_32(iterator) {
		Ok(val) => Ok(val as i32),
		Err(e) => exception_wrap!(Err(e), "While reading signed 32 bit MP integer")
	}
}

pub fn read_sint_64(iterator: &mut CustomIterator) -> EhResult<i64> {
	match read_int_64(iterator) {
		Ok(val) => Ok(val as i64),
		Err(e) => exception_wrap!(Err(e), "While reading signed 64 bit MP integer")
	}
}

pub fn read_int_auto(iterator: &mut CustomIterator) -> EhResult<u32> {
	let type_fml = exception_wrap!(iterator.next(), "While reading MP integer type")?;
	match type_fml {
		0..=0x80 => {
			Ok(type_fml as u32)
		}
		0xCC => {
			Ok(exception_wrap!(read_int_8(iterator), "While automatically reading a 8 bit integer")? as u32)
		}
		0xCD => {
			Ok(exception_wrap!(read_int_16(iterator), "While automatically reading a 16 bit integer")? as u32)
		}
		0xCE => {
			Ok(exception_wrap!(read_int_32(iterator), "While automatically reading a 32 bit integer")? as u32)
		}
		_ => {
			exception!("While expecting MP integer type, got: ", format!("0x{:X}", type_fml))
		}
	}
}

//Float:

pub fn read_float_32(iterator: &mut CustomIterator) -> EhResult<f32> {
	match read_int_32(iterator) {
		Ok(val) => Ok(f32::from_bits(val)),
		Err(e) => exception_wrap!(Err(e), "While reading 32 bit MP float")
	}
}

pub fn read_float_64(iterator: &mut CustomIterator) -> EhResult<f64> {
	match read_int_64(iterator) {
		Ok(val) => Ok(val as f64),
		Err(e) => exception_wrap!(Err(e), "While reading 64 bit MP float")
	}
}

//Map:

pub fn read_map_flex(iterator: &mut CustomIterator) -> EhResult<u32> {
	let next = exception_wrap!(iterator.next(), "While reading MP flex map type/value")?;
	Ok((next as u32) - 0x80)
}

pub fn read_map_16(iterator: &mut CustomIterator) -> EhResult<u16> {
	exception_wrap!(read_int_16(iterator), "While reading MP 16-bit length prefixed map length")
}

pub fn read_map_32(iterator: &mut CustomIterator) -> EhResult<u32> {
	exception_wrap!(read_int_32(iterator), "While reading MP 32-bit length prefixed map length")
}

pub fn read_map_auto(iterator: &mut CustomIterator) -> EhResult<u32> {
	let type_fml = exception_wrap!(iterator.peek(), "While reading MP map type")?;
	match type_fml {
		0x80..=0x91 => {
			Ok(exception_wrap!(read_map_flex(iterator), "While automatically reading MP flex map")? as u32)
		}
		0xDE => {
			iterator.skip();
			Ok(exception_wrap!(read_int_16(iterator), "While automatically reading 16 MP bit map")? as u32)
		}
		0xDF => {
			iterator.skip();
			Ok(exception_wrap!(read_int_32(iterator), "While automatically reading 32 MP bit map")? as u32)
		}
		_ => {
			exception!("While expecting MP map type, got: ", format!("0x{:X}", type_fml))
		}
	}
}

//Array:

pub fn read_array_flex(iterator: &mut CustomIterator) -> EhResult<u8> {
	let next = exception_wrap!(iterator.next(), "While reading MP flex array type")?;
	Ok(next - 0x90)
}

pub fn read_array_16(iterator: &mut CustomIterator) -> EhResult<u16> {
	exception_wrap!(read_int_16(iterator), "While reading MP 16-bit length prefixed array length")
}

pub fn read_array_32(iterator: &mut CustomIterator) -> EhResult<u32> {
	exception_wrap!(read_int_32(iterator), "While reading MP 32-bit length prefixed array length")
}

pub fn read_array_auto(iterator: &mut CustomIterator) -> EhResult<u32> {
	let type_fml = exception_wrap!(iterator.peek(), "While reading MP array type")?;
	match type_fml {
		0x90..=0xA1 => {
			Ok(exception_wrap!(read_array_flex(iterator), "While automatically reading MP flex array")? as u32)
		}
		0xDC => {
			iterator.skip();
			Ok(exception_wrap!(read_int_16(iterator), "While automatically reading 16 bit MP array")? as u32)
		}
		0xDD => {
			iterator.skip();
			Ok(exception_wrap!(read_int_32(iterator), "While automatically reading 32 bit MP array")? as u32)
		}
		_ => {
			exception!("While expecting MP array type, got: ", format!("0x{:X}", type_fml))
		}
	}
}

//String:

fn read_string_len(iterator: &mut CustomIterator, length: usize) -> EhResult<String> {
	let bytes = exception_wrap!(iterator.read_bytes(length), "While reading fixed length MP string")?;
	exception_from!(String::from_utf8(bytes), "While converting fixed length MP string bytes")
}

pub fn read_string_flex(iterator: &mut CustomIterator) -> EhResult<String> {
	let next = exception_wrap!(iterator.next(), "While reading MP flex string length")?;
	let length = (next - 0xA0) as usize;
	exception_wrap!(read_string_len(iterator, length), "While reading MP flex string")
}

pub fn read_string_8(iterator: &mut CustomIterator) -> EhResult<String> {
	let length = exception_wrap!(read_int_8(iterator), "While reading an 8 bit string prefix")? as usize;
	read_string_len(iterator, length)
}

pub fn read_string_16(iterator: &mut CustomIterator) -> EhResult<String> {
	let length = exception_wrap!(read_int_16(iterator), "While reading a 16 bit string prefix")? as usize;
	read_string_len(iterator, length)
}

pub fn read_string_32(iterator: &mut CustomIterator) -> EhResult<String> {
	let length = exception_wrap!(read_int_32(iterator), "While reading a 32 bit string prefix")? as usize;
	read_string_len(iterator, length)
}

pub fn read_string_auto(iterator: &mut CustomIterator) -> EhResult<Option<String>> {
	let type_fml = exception_wrap!(iterator.peek(), "While reading MP string")?;
	match type_fml {
		0xA0..=0xBF => {
			Ok(Some(exception_wrap!(read_string_flex(iterator), "While automatically reading a flex string")?))
		}
		0xC0 => {
			iterator.skip();
			Ok(None)
		}
		0xD9 => {
			iterator.skip();
			Ok(Some(exception_wrap!(read_string_8(iterator), "While automatically reading a 8 bit prefixed length string")?))
		}
		0xDA => {
			iterator.skip();
			Ok(Some(exception_wrap!(read_string_16(iterator), "While automatically reading a 16 bit prefixed length string")?))
		}
		_ => {
			exception!("While expecting MP string type, got: ", format!("0x{:X}", type_fml))
		}
	}
}

//Boolean:

pub fn read_bool_auto(iterator: &mut CustomIterator) -> EhResult<bool> {
	let type_fml = exception_wrap!(iterator.next(), "While reading MP bool type/value")?;
	match type_fml {
		0xC2 => Ok(false),
		0xC3 => Ok(true),
		_ => exception!("While expecting MP boolean type, got: ", format!("0x{:X}", type_fml))
	}
}

//Binary:

pub fn read_binary_len(iterator: &mut CustomIterator, length: usize) -> EhResult<Vec<u8>> {
	exception_wrap!(iterator.read_bytes(length), "While reading MP binary bytes")
}

pub fn read_binary_8(iterator: &mut CustomIterator) -> EhResult<Vec<u8>> {
	let length = exception_wrap!(read_int_8(iterator), "While reading 8 bit length prefix MP binary")? as usize;
	read_binary_len(iterator, length)
}

pub fn read_binary_16(iterator: &mut CustomIterator) -> EhResult<Vec<u8>> {
	let length = exception_wrap!(read_int_16(iterator), "While reading 16 bit length prefix MP binary")? as usize;
	read_binary_len(iterator, length)
}

pub fn read_binary_32(iterator: &mut CustomIterator) -> EhResult<Vec<u8>> {
	let length = exception_wrap!(read_int_32(iterator), "While reading 32 bit length prefix MP binary")? as usize;
	read_binary_len(iterator, length)
}

pub fn read_binary_auto(iterator: &mut CustomIterator) -> EhResult<Option<Vec<u8>>> {
	let type_fml = exception_wrap!(iterator.next(), "While reading MP binary type")?;
	match type_fml {
		0xC0 => {
			Ok(None)
		}
		0xC4 => {
			Ok(Some(exception_wrap!(read_binary_8(iterator), "While automatically reading 8 bit MP binary")?))
		}
		0xC5 => {
			Ok(Some(exception_wrap!(read_binary_16(iterator), "While automatically reading 16 bit MP binary")?))
		}
		0xC6 => {
			Ok(Some(exception_wrap!(read_binary_32(iterator), "While automatically reading 32 bit MP binary")?))
		}
		_ => {
			exception!("While expecting MP binary type, got: ", format!("0x{:X}", type_fml))
		}
	}
}
