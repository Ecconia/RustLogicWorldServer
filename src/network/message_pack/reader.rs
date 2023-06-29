use crate::prelude::*;

use crate::util::custom_iterator::CustomIterator;

//### Internal helpers: ##############

//Macros / Helpers:

macro_rules! expect_type {
	($iterator:expr, $name:expr, $value:expr) => {
		let value = exception_wrap!($iterator.next(), "While reading MP ", $name, " type byte")?;
		if value != $value {
			return exception!("Expected MP ", $name, ", but got type byte: ", value);
		}
	}
}

macro_rules! _optional {
	($iterator:expr, $input:expr) => {
		match exception_wrap!($crate::network::message_pack::reader::check_null($iterator), "While probing for ", "Nil", " value")? {
			true => Ok(None),
			false => Ok(Some($input?)),
		}
	}
}
pub(crate) use _optional as optional;

pub fn check_null(iterator: &mut CustomIterator) -> EhResult<bool> {
	let value = exception_wrap!(iterator.peek(), "While checking for ", "Nil", " type byte")?;
	if value == 0xC0 {
		iterator.skip(); //Drop the null byte.
		return Ok(true);
	}
	Ok(false)
}

//String / Bytes:

fn read_string_len(iterator: &mut CustomIterator, length: usize) -> EhResult<String> {
	let bytes = exception_wrap!(iterator.read_bytes(length), "While reading fixed length MP string")?;
	exception_from!(String::from_utf8(bytes), "While converting fixed length MP string bytes to UTF8")
}

fn read_bytes_len(iterator: &mut CustomIterator, length: usize) -> EhResult<Vec<u8>> {
	exception_wrap!(iterator.read_bytes(length), "While reading MP binary bytes")
}

//### Explicit types: ################

//Positive/Unsigned Integers:

pub fn read_pos_int_fix(iterator: &mut CustomIterator) -> EhResult<u8> {
	let value = exception_wrap!(iterator.next(), "While reading MP ", "fix int")?;
	if value >= 0x80 {
		return exception!("Expected MP ", "fix int", ", but got value: ", value);
	}
	Ok(value)
}

pub fn read_neg_int_fix(iterator: &mut CustomIterator) -> EhResult<i8> {
	let value = exception_wrap!(iterator.next(), "While reading MP ", "fix int")?;
	if value < 0xE0 {
		return exception!("Expected MP ", "fix int", ", but got value: ", value);
	}
	Ok(value as i8) //Should make the number negative
}

//Unsigned Integers:

pub fn read_int_8(iterator: &mut CustomIterator) -> EhResult<u8> {
	expect_type!(iterator, "unsigned int 8", 0xCC);
	exception_wrap!(iterator.next(), "While reading MP ", "unsigned int 8")
}

pub fn read_int_16(iterator: &mut CustomIterator) -> EhResult<u16> {
	expect_type!(iterator, "unsigned int 16", 0xCD);
	exception_wrap!(iterator.read_be_u16(), "While reading MP ", "unsigned int 16")
}

pub fn read_int_32(iterator: &mut CustomIterator) -> EhResult<u32> {
	expect_type!(iterator, "unsigned int 32", 0xCE);
	exception_wrap!(iterator.read_be_u32(), "While reading MP ", "unsigned int 32")
}

pub fn read_int_64(iterator: &mut CustomIterator) -> EhResult<u64> {
	expect_type!(iterator, "unsigned int 64", 0xCF);
	exception_wrap!(iterator.read_be_u64(), "While reading MP ", "unsigned int 64")
}

//Signed Integers:

pub fn read_s_int_8(iterator: &mut CustomIterator) -> EhResult<i8> {
	expect_type!(iterator, "signed int 8", 0xD0);
	Ok(exception_wrap!(iterator.next(), "While reading MP ", "unsigned int 8")? as i8)
}

pub fn read_s_int_16(iterator: &mut CustomIterator) -> EhResult<i16> {
	expect_type!(iterator, "signed int 16", 0xD1);
	exception_wrap!(iterator.read_be_i16(), "While reading MP ", "signed int 16")
}

pub fn read_s_int_32(iterator: &mut CustomIterator) -> EhResult<i32> {
	expect_type!(iterator, "signed int 32", 0xD2);
	exception_wrap!(iterator.read_be_i32(), "While reading MP ", "signed int 32")
}

pub fn read_s_int_64(iterator: &mut CustomIterator) -> EhResult<i64> {
	expect_type!(iterator, "signed int 64", 0xD3);
	exception_wrap!(iterator.read_be_i64(), "While reading MP ", "signed int 64")
}

//Map:

pub fn read_map_fix(iterator: &mut CustomIterator) -> EhResult<u8> {
	let value = exception_wrap!(iterator.next(), "While reading MP ", "fix map")?;
	if !(0x80..0x90).contains(&value) {
		return exception!("Expected MP ", "fix map", ", but got value: ", value);
	}
	Ok(value - 0x80)
}

pub fn read_map_16(iterator: &mut CustomIterator) -> EhResult<u16> {
	expect_type!(iterator, "map 16", 0xDE);
	exception_wrap!(iterator.read_be_u16(), "While reading MP ", "map 16")
}

pub fn read_map_32(iterator: &mut CustomIterator) -> EhResult<u32> {
	expect_type!(iterator, "map 32", 0xDF);
	exception_wrap!(iterator.read_be_u32(), "While reading MP ", "map 32")
}

//Array:

pub fn read_array_fix(iterator: &mut CustomIterator) -> EhResult<u8> {
	let value = exception_wrap!(iterator.next(), "While reading MP ", "fix array")?;
	if !(0x90..0xA0).contains(&value) {
		return exception!("Expected MP ", "fix array", ", but got value: ", value);
	}
	Ok(value - 0x90)
}

pub fn read_array_16(iterator: &mut CustomIterator) -> EhResult<u16> {
	expect_type!(iterator, "array 16", 0xDC);
	exception_wrap!(iterator.read_be_u16(), "While reading MP ", "array 16")
}

pub fn read_array_32(iterator: &mut CustomIterator) -> EhResult<u32> {
	expect_type!(iterator, "array 32", 0xDD);
	exception_wrap!(iterator.read_be_u32(), "While reading MP ", "array 32")
}

//String:

pub fn read_string_fix(iterator: &mut CustomIterator) -> EhResult<String> {
	let value = exception_wrap!(iterator.next(), "While reading MP ", "fix string", " type/length")?;
	if value < 0xA0 || value >= 0xC0 {
		return exception!("Expected MP ", "fix string", ", but got value: ", value);
	}
	exception_wrap!(read_string_len(iterator, (value - 0xA0) as usize), "While reading MP ", "fix string")
}

pub fn read_string_8(iterator: &mut CustomIterator) -> EhResult<String> {
	expect_type!(iterator, "string 8", 0xD9);
	let length = exception_wrap!(iterator.next(), "While reading ", "string 8", " length")?;
	exception_wrap!(read_string_len(iterator, length as usize), "While reading MP ", "string 8")
}

pub fn read_string_16(iterator: &mut CustomIterator) -> EhResult<String> {
	expect_type!(iterator, "string 16", 0xDA);
	let length = exception_wrap!(iterator.read_be_u16(), "While reading ", "string 16", " length")?;
	exception_wrap!(read_string_len(iterator, length as usize), "While reading MP ", "string 16")
}

pub fn read_string_32(iterator: &mut CustomIterator) -> EhResult<String> {
	expect_type!(iterator, "string 32", 0xDB);
	let length = exception_wrap!(iterator.read_be_u32(), "While reading ", "string 32", " length")?;
	exception_wrap!(read_string_len(iterator, length as usize), "While reading MP ", "string 32")
}

//Bytes:

pub fn read_binary_8(iterator: &mut CustomIterator) -> EhResult<Vec<u8>> {
	expect_type!(iterator, "binary 8", 0xC4);
	let length = exception_wrap!(iterator.next(), "While reading MP ", "binary 8", " length")? as usize;
	exception_wrap!(read_bytes_len(iterator, length), "While reading MP ", "binary 8", " bytes")
}

pub fn read_binary_16(iterator: &mut CustomIterator) -> EhResult<Vec<u8>> {
	expect_type!(iterator, "binary 16", 0xC5);
	let length = exception_wrap!(iterator.read_be_u16(), "While reading MP ", "binary 16", " length")? as usize;
	exception_wrap!(read_bytes_len(iterator, length), "While reading MP ", "binary 16", " bytes")
}

pub fn read_binary_32(iterator: &mut CustomIterator) -> EhResult<Vec<u8>> {
	expect_type!(iterator, "binary 32", 0xC6);
	let length = exception_wrap!(iterator.read_be_u32(), "While reading MP ", "binary 32", " length")? as usize;
	exception_wrap!(read_bytes_len(iterator, length), "While reading MP ", "binary 32", " bytes")
}

//Float:

pub fn read_float_32(iterator: &mut CustomIterator) -> EhResult<f32> {
	expect_type!(iterator, "float 32", 0xCA);
	exception_wrap!(iterator.read_be_f32(), "While reading MP ", "float 32")
}

pub fn read_float_64(iterator: &mut CustomIterator) -> EhResult<f64> {
	expect_type!(iterator, "float 64", 0xCB);
	exception_wrap!(iterator.read_be_f64(), "While reading MP ", "float 64")
}

//### Implicit types: ################

pub fn read_u8(iterator: &mut CustomIterator) -> EhResult<u8> {
	let type_byte = exception_wrap!(iterator.next(), "While reading ", "u8", " via MP: ", "type")?;
	match type_byte {
		0..=0x7F => {
			Ok(type_byte)
		},
		0xCC => {
			exception_wrap!(iterator.next(), "While reading ", "u8", " via MP: ", "byte")
		},
		_ => {
			exception!("Expected MP type, that would fit ", "u8", ", but got: ", type_byte)
		}
	}
}

pub fn read_u16(iterator: &mut CustomIterator) -> EhResult<u16> {
	let type_byte = exception_wrap!(iterator.next(), "While reading ", "u16", " via MP: ", "type")?;
	match type_byte {
		0..=0x7F => {
			Ok(type_byte as u16)
		},
		0xCC => {
			Ok(exception_wrap!(iterator.next(), "While reading ", "u16", " via MP: ", "ubyte")? as u16)
		},
		0xCD => {
			exception_wrap!(iterator.read_be_u16(), "While reading ", "u16", " via MP: ", "ushort")
		},
		_ => {
			exception!("Expected MP type, that would fit ", "u16", ", but got: ", type_byte)
		}
	}
}

pub fn read_u32(iterator: &mut CustomIterator) -> EhResult<u32> {
	let type_byte = exception_wrap!(iterator.next(), "While reading ", "u32", " via MP: ", "type")?;
	match type_byte {
		0..=0x7F => {
			Ok(type_byte as u32)
		},
		0xCC => {
			Ok(exception_wrap!(iterator.next(), "While reading ", "u32", " via MP: ", "ubyte")? as u32)
		},
		0xCD => {
			Ok(exception_wrap!(iterator.read_be_u16(), "While reading ", "u32", " via MP: ", "ushort")? as u32)
		},
		0xCE => {
			exception_wrap!(iterator.read_be_u32(), "While reading ", "u32", " via MP: ", "uint")
		},
		_ => {
			exception!("Expected MP type, that would fit ", "u32", ", but got: ", format!("0x{:X}", type_byte))
		}
	}
}

pub fn read_u64(iterator: &mut CustomIterator) -> EhResult<u64> {
	let type_byte = exception_wrap!(iterator.next(), "While reading ", "u64", " via MP: ", "type")?;
	match type_byte {
		0..=0x7F => {
			Ok(type_byte as u64)
		},
		0xCC => {
			Ok(exception_wrap!(iterator.next(), "While reading ", "u64", " via MP: ", "ubyte")? as u64)
		},
		0xCD => {
			Ok(exception_wrap!(iterator.read_be_u16(), "While reading ", "u64", " via MP: ", "ushort")? as u64)
		},
		0xCE => {
			Ok(exception_wrap!(iterator.read_be_u32(), "While reading ", "u64", " via MP: ", "uint")? as u64)
		},
		0xCF => {
			exception_wrap!(iterator.read_be_u64(), "While reading ", "u64", " via MP: ", "ulong")
		},
		_ => {
			exception!("Expected MP type, that would fit ", "u64", ", but got: ", type_byte)
		}
	}
}

pub fn read_i8(iterator: &mut CustomIterator) -> EhResult<i8> {
	let type_byte = exception_wrap!(iterator.next(), "While reading ", "i8", " via MP: ", "type")?;
	match type_byte {
		0..=0x7F => {
			//Positive fix - fits
			Ok(type_byte as i8) //Although positive, this fits in a signed byte!
		},
		0xE0..=0xFF => {
			//Negative fix - fits
			Ok(type_byte as i8)
		},
		0xD0 => {
			//Negative byte - fits
			Ok(exception_wrap!(iterator.next(), "While reading ", "i8", " via MP: ", "sbyte")? as i8)
		},
		0xCC => {
			//Positive byte - fits, if the highest bit is not set!
			let value = exception_wrap!(iterator.next(), "While reading ", "i8", " via MP: ", "byte")?;
			if value >= 0x80 {
				return exception!("Expected ", "signed byte", ", but got unsigned byte with highest bit set.");
			}
			Ok(value as i8)
		},
		_ => {
			exception!("Expected MP type, that would fit ", "i8", ", but got: ", type_byte)
		}
	}
}

pub fn read_i16(iterator: &mut CustomIterator) -> EhResult<i16> {
	let type_byte = exception_wrap!(iterator.next(), "While reading ", "i16", " via MP: ", "type")?;
	match type_byte {
		0..=0x7F => {
			//Positive fix - fits
			Ok(type_byte as i16)
		},
		0xE0..=0xFF => {
			//Negative fix - fits
			Ok(type_byte as i16)
		},
		0xD0 => {
			//Negative byte - fits
			Ok(exception_wrap!(iterator.next(), "While reading ", "i16", " via MP: ", "sbyte")? as i16)
		},
		0xCC => {
			//Positive byte - fits
			Ok(exception_wrap!(iterator.next(), "While reading ", "i16", " via MP: ", "byte")? as i16)
		},
		0xD1 => {
			//Negative short - fits
			exception_wrap!(iterator.read_be_i16(), "While reading ", "i16", " via MP: ", "sshort")
		}
		0xCD => {
			//Positive short - fits, if the highest bit is not set!
			let value = exception_wrap!(iterator.read_be_i16(), "While reading ", "i16", " via MP: ", "short")?;
			if value < 0 {
				return exception!("Expected ", "signed short", ", but got unsigned short with highest bit set.");
			}
			Ok(value)
		}
		_ => {
			exception!("Expected MP type, that would fit ", "i16", ", but got: ", type_byte)
		}
	}
}

pub fn read_i32(iterator: &mut CustomIterator) -> EhResult<i32> {
	let type_byte = exception_wrap!(iterator.next(), "While reading ", "i32", " via MP: ", "type")?;
	match type_byte {
		0..=0x7F => {
			//Positive fix - fits
			Ok(type_byte as i32)
		},
		0xE0..=0xFF => {
			//Negative fix - fits
			Ok(type_byte as i32)
		},
		0xD0 => {
			//Negative byte - fits
			Ok(exception_wrap!(iterator.next(), "While reading ", "i32", " via MP: ", "sbyte")? as i32)
		},
		0xCC => {
			//Positive byte - fits
			Ok(exception_wrap!(iterator.next(), "While reading ", "i32", " via MP: ", "byte")? as i32)
		},
		0xD1 => {
			//Negative short - fits
			Ok(exception_wrap!(iterator.read_be_i16(), "While reading ", "i32", " via MP: ", "sshort")? as i32)
		},
		0xCD => {
			//Positive short - fits
			Ok(exception_wrap!(iterator.read_be_i16(), "While reading ", "i32", " via MP: ", "short")? as i32)
		},
		0xD2 => {
			//Negative int - fits
			exception_wrap!(iterator.read_be_i32(), "While reading ", "i32", " via MP: ", "sint")
		},
		0xCE => {
			//Positive int - fits, if the highest bit is not set!
			let value = exception_wrap!(iterator.read_be_i32(), "While reading ", "i32", " via MP: ", "int")?;
			if value < 0 {
				return exception!("Expected ", "signed int", ", but got unsigned int with highest bit set.");
			}
			Ok(value)
		},
		_ => {
			exception!("Expected MP type, that would fit ", "i32", ", but got: ", type_byte)
		}
	}
}

pub fn read_i64(iterator: &mut CustomIterator) -> EhResult<i64> {
	let type_byte = exception_wrap!(iterator.next(), "While reading ", "i64", " via MP: ", "type")?;
	match type_byte {
		0..=0x7F => {
			//Positive fix - fits
			Ok(type_byte as i64)
		},
		0xE0..=0xFF => {
			//Negative fix - fits
			Ok(type_byte as i64)
		},
		0xD0 => {
			//Negative byte - fits
			Ok(exception_wrap!(iterator.next(), "While reading ", "i64", " via MP: ", "sbyte")? as i64)
		},
		0xCC => {
			//Positive byte - fits
			Ok(exception_wrap!(iterator.next(), "While reading ", "i64", " via MP: ", "byte")? as i64)
		},
		0xD1 => {
			//Negative short - fits
			Ok(exception_wrap!(iterator.read_be_i16(), "While reading ", "i64", " via MP: ", "sshort")? as i64)
		},
		0xCD => {
			//Positive short - fits
			Ok(exception_wrap!(iterator.read_be_i16(), "While reading ", "i64", " via MP: ", "short")? as i64)
		},
		0xD2 => {
			//Negative int - fits
			Ok(exception_wrap!(iterator.read_be_i32(), "While reading ", "i64", " via MP: ", "sint")? as i64)
		},
		0xCE => {
			//Positive int - fits
			Ok(exception_wrap!(iterator.read_be_i32(), "While reading ", "i64", " via MP: ", "int")? as i64)
		},
		0xD3 => {
			//Negative long - fits
			exception_wrap!(iterator.read_be_i64(), "While reading ", "i64", " via MP: ", "slong")
		},
		0xCF => {
			//Positive long - fits, if the highest bit is not set!
			let value = exception_wrap!(iterator.read_be_i64(), "While reading ", "i64", " via MP: ", "long")?;
			if value < 0 {
				return exception!("Expected ", "signed long", ", but got unsigned long with highest bit set.");
			}
			Ok(value)
		},
		_ => {
			exception!("Expected MP type, that would fit ", "i64", ", but got: ", type_byte)
		}
	}
}

pub fn read_f32(iterator: &mut CustomIterator) -> EhResult<f32> {
	let type_byte = exception_wrap!(iterator.next(), "While reading ", "f32", " via MP: ", "type")?;
	match type_byte {
		0xCA => {
			exception_wrap!(iterator.read_be_f32(), "While reading ", "f32", " via MP")
		},
		_ => {
			exception!("Expected MP type, that would fit ", "f32", ", but got: ", type_byte)
		}
	}
}

pub fn read_f64(iterator: &mut CustomIterator) -> EhResult<f64> {
	let type_byte = exception_wrap!(iterator.next(), "While reading ", "f64", " via MP: ", "type")?;
	match type_byte {
		0xCB => {
			exception_wrap!(iterator.read_be_f64(), "While reading ", "f64", " via MP")
		},
		_ => {
			exception!("Expected MP type, that would fit ", "f64", ", but got: ", type_byte)
		}
	}
}

pub fn read_string(iterator: &mut CustomIterator) -> EhResult<String> {
	let type_byte = exception_wrap!(iterator.next(), "While reading ", "String", " via MP: ", "type")?;
	match type_byte {
		0xA0..=0xBF => {
			exception_wrap!(read_string_len(iterator, (type_byte - 0xA0) as usize), "While reading MP ", "String", " via MP: ", "fix")
		},
		0xD9 => {
			let length = exception_wrap!(iterator.next(), "While reading ", "String", " via MP: ", "8 length")?;
			exception_wrap!(read_string_len(iterator, length as usize), "While reading MP ", "String", " via MP: ", "8")
		},
		0xDA => {
			let length = exception_wrap!(iterator.read_be_u16(), "While reading ", "String", " via MP: ", "16 length")?;
			exception_wrap!(read_string_len(iterator, length as usize), "While reading MP ", "String", " via MP: ", "16")
		},
		0xDB => {
			let length = exception_wrap!(iterator.read_be_u32(), "While reading ", "String", " via MP: ", "32 length")?;
			exception_wrap!(read_string_len(iterator, length as usize), "While reading MP ", "String", " via MP: ", "32")
		},
		_ => {
			exception!("Expected MP type, that would fit ", "String", ", but got: ", type_byte)
		}
	}
}

pub fn read_bool(iterator: &mut CustomIterator) -> EhResult<bool> {
	let type_byte = exception_wrap!(iterator.next(), "While reading ", "bool", " via MP: ", "type")?;
	match type_byte {
		0xC2 => Ok(false),
		0xC3 => Ok(true),
		_ => {
			exception!("Expected MP type, that would fit ", "bool", ", but got: ", type_byte)
		}
	}
}

//Use the maximum value u32 as return type.
pub fn read_array(iterator: &mut CustomIterator) -> EhResult<u32> {
	let type_byte = exception_wrap!(iterator.next(), "While reading ", "array", " via MP: ", "type")?;
	match type_byte {
		0x90..=0x9F => {
			Ok((type_byte - 0x90) as u32)
		}
		0xDC => {
			Ok(exception_wrap!(iterator.read_be_u16(), "While reading ", "array", " via MP: ", "array 16")? as u32)
		}
		0xDD => {
			exception_wrap!(iterator.read_be_u32(), "While reading ", "array", " via MP: ", "array 32")
		}
		_ => {
			exception!("Expected MP type, that would fit ", "array", ", but got: ", type_byte)
		}
	}
}

//Use the maximum value u32 as return type.
pub fn try_array(iterator: &mut CustomIterator) -> EhResult<Option<u32>> {
	let type_byte = exception_wrap!(iterator.next(), "While trying ", "array", " via MP: ", "type")?;
	Ok(match type_byte {
		0x90..=0x9F => {
			Some((type_byte - 0x90) as u32)
		}
		0xDC => {
			Some(exception_wrap!(iterator.read_be_u16(), "While reading ", "array", " via MP: ", "array 16")? as u32)
		}
		0xDD => {
			Some(exception_wrap!(iterator.read_be_u32(), "While reading ", "array", " via MP: ", "array 32")?)
		}
		_ => {
			None
		}
	})
}

//Use the maximum value u32 as return type.
pub fn read_map(iterator: &mut CustomIterator) -> EhResult<u32> {
	let type_byte = exception_wrap!(iterator.next(), "While reading ", "map", " via MP: ", "type")?;
	match type_byte {
		0x80..=0x8F => {
			Ok((type_byte - 0x80) as u32)
		}
		0xDE => {
			Ok(exception_wrap!(iterator.read_be_u16(), "While reading ", "map", " via MP: ", "map 16")? as u32)
		}
		0xDF => {
			exception_wrap!(iterator.read_be_u32(), "While reading ", "map", " via MP: ", "map 32")
		}
		_ => {
			exception!("Expected MP type, that would fit ", "map", ", but got: ", type_byte)
		}
	}
}

pub fn read_bytes(iterator: &mut CustomIterator) -> EhResult<Vec<u8>> {
	let type_byte = exception_wrap!(iterator.next(), "While reading ", "bytes", " via MP: ", "type")?;
	match type_byte {
		0xC4 => {
			let length = exception_wrap!(iterator.next(), "While reading ", "bytes", " via MP: ", "8 length")? as usize;
			exception_wrap!(read_bytes_len(iterator, length), "While reading ", "bytes", " via MP: ", "8 bytes")
		}
		0xC5 => {
			let length = exception_wrap!(iterator.read_be_u16(), "While reading ", "bytes", " via MP: ", "8 length")? as usize;
			exception_wrap!(read_bytes_len(iterator, length), "While reading ", "bytes", " via MP: ", "8 bytes")
		}
		0xC6 => {
			let length = exception_wrap!(iterator.read_be_u32(), "While reading ", "bytes", " via MP: ", "8 length")? as usize;
			exception_wrap!(read_bytes_len(iterator, length), "While reading ", "bytes", " via MP: ", "8 bytes")
		}
		_ => {
			exception!("Expected MP type, that would fit ", "bytes", ", but got: ", type_byte)
		}
	}
}

pub fn read_ext(iterator: &mut CustomIterator) -> EhResult<(u8, Vec<u8>)> {
	let type_byte = exception_wrap!(iterator.next(), "While reading ", "ext", " via MP: ", "type")?;
	match type_byte {
		0xC7 => {
			//Ext 8
			let length = exception_wrap!(iterator.next(), "While reading ", "ext", "via MP: ", "Ext8 length")?;
			let sub_type = exception_wrap!(iterator.next(), "While reading ", "ext", "via MP: ", "Ext8 type")?;
			let data = exception_wrap!(iterator.read_bytes(length as usize), "While reading ", "ext", "via MP: ", "Ext8 bytes")?;
			Ok((sub_type, data))
		}
		0xC8 => {
			//Ext 16
			let length = exception_wrap!(iterator.read_be_u16(), "While reading ", "ext", "via MP: ", "Ext16 length")?;
			let sub_type = exception_wrap!(iterator.next(), "While reading ", "ext", "via MP: ", "Ext16 type")?;
			let data = exception_wrap!(iterator.read_bytes(length as usize), "While reading ", "ext", "via MP: ", "Ext16 bytes")?;
			Ok((sub_type, data))
		}
		0xC9 => {
			//Ext 32
			let length = exception_wrap!(iterator.read_be_u32(), "While reading ", "ext", "via MP: ", "Ext32 length")?;
			let sub_type = exception_wrap!(iterator.next(), "While reading ", "ext", "via MP: ", "Ext32 type")?;
			let data = exception_wrap!(iterator.read_bytes(length as usize), "While reading ", "ext", "via MP: ", "Ext32 bytes")?;
			Ok((sub_type, data))
		}
		0xD4 => {
			//Fix Ext 1
			let sub_type = exception_wrap!(iterator.next(), "While reading ", "ext", "via MP: ", "FixExt1 type")?;
			let data = exception_wrap!(iterator.read_bytes(1), "While reading ", "ext", "via MP: ", "FixExt1 bytes")?;
			Ok((sub_type, data))
		}
		0xD5 => {
			//Fix Ext 2
			let sub_type = exception_wrap!(iterator.next(), "While reading ", "ext", "via MP: ", "FixExt2 type")?;
			let data = exception_wrap!(iterator.read_bytes(2), "While reading ", "ext", "via MP: ", "FixExt2 bytes")?;
			Ok((sub_type, data))
		}
		0xD6 => {
			//Fix Ext 4
			let sub_type = exception_wrap!(iterator.next(), "While reading ", "ext", "via MP: ", "FixExt4 type")?;
			let data = exception_wrap!(iterator.read_bytes(4), "While reading ", "ext", "via MP: ", "FixExt4 bytes")?;
			Ok((sub_type, data))
		}
		0xD7 => {
			//Fix Ext 8
			let sub_type = exception_wrap!(iterator.next(), "While reading ", "ext", "via MP: ", "FixExt8 type")?;
			let data = exception_wrap!(iterator.read_bytes(8), "While reading ", "ext", "via MP: ", "FixExt8 bytes")?;
			Ok((sub_type, data))
		}
		0xD8 => {
			//Fix Ext 16
			let sub_type = exception_wrap!(iterator.next(), "While reading ", "ext", "via MP: ", "FixExt16 type")?;
			let data = exception_wrap!(iterator.read_bytes(16), "While reading ", "ext", "via MP: ", "FixExt16 bytes")?;
			Ok((sub_type, data))
		}
		_ => {
			exception!("Expected MP type, that would fit ", "ext", ", but got: ", type_byte)
		}
	}
}

pub fn try_ext(iterator: &mut CustomIterator) -> EhResult<Option<(u8, Vec<u8>)>> {
	let type_byte = exception_wrap!(iterator.next(), "While trying ", "ext", " via MP: ", "type")?;
	Ok(match type_byte {
		0xC7 => {
			//Ext 8
			let length = exception_wrap!(iterator.next(), "While trying ", "ext", "via MP: ", "Ext8 length")?;
			let sub_type = exception_wrap!(iterator.next(), "While trying ", "ext", "via MP: ", "Ext8 type")?;
			let data = exception_wrap!(iterator.read_bytes(length as usize), "While trying ", "ext", "via MP: ", "Ext8 bytes")?;
			Some((sub_type, data))
		}
		0xC8 => {
			//Ext 16
			let length = exception_wrap!(iterator.read_be_u16(), "While trying ", "ext", "via MP: ", "Ext16 length")?;
			let sub_type = exception_wrap!(iterator.next(), "While trying ", "ext", "via MP: ", "Ext16 type")?;
			let data = exception_wrap!(iterator.read_bytes(length as usize), "While trying ", "ext", "via MP: ", "Ext16 bytes")?;
			Some((sub_type, data))
		}
		0xC9 => {
			//Ext 32
			let length = exception_wrap!(iterator.read_be_u32(), "While trying ", "ext", "via MP: ", "Ext32 length")?;
			let sub_type = exception_wrap!(iterator.next(), "While trying ", "ext", "via MP: ", "Ext32 type")?;
			let data = exception_wrap!(iterator.read_bytes(length as usize), "While trying ", "ext", "via MP: ", "Ext32 bytes")?;
			Some((sub_type, data))
		}
		0xD4 => {
			//Fix Ext 1
			let sub_type = exception_wrap!(iterator.next(), "While trying ", "ext", "via MP: ", "FixExt1 type")?;
			let data = exception_wrap!(iterator.read_bytes(1), "While trying ", "ext", "via MP: ", "FixExt1 bytes")?;
			Some((sub_type, data))
		}
		0xD5 => {
			//Fix Ext 2
			let sub_type = exception_wrap!(iterator.next(), "While trying ", "ext", "via MP: ", "FixExt2 type")?;
			let data = exception_wrap!(iterator.read_bytes(2), "While trying ", "ext", "via MP: ", "FixExt2 bytes")?;
			Some((sub_type, data))
		}
		0xD6 => {
			//Fix Ext 4
			let sub_type = exception_wrap!(iterator.next(), "While trying ", "ext", "via MP: ", "FixExt4 type")?;
			let data = exception_wrap!(iterator.read_bytes(4), "While trying ", "ext", "via MP: ", "FixExt4 bytes")?;
			Some((sub_type, data))
		}
		0xD7 => {
			//Fix Ext 8
			let sub_type = exception_wrap!(iterator.next(), "While trying ", "ext", "via MP: ", "FixExt8 type")?;
			let data = exception_wrap!(iterator.read_bytes(8), "While trying ", "ext", "via MP: ", "FixExt8 bytes")?;
			Some((sub_type, data))
		}
		0xD8 => {
			//Fix Ext 16
			let sub_type = exception_wrap!(iterator.next(), "While trying ", "ext", "via MP: ", "FixExt16 type")?;
			let data = exception_wrap!(iterator.read_bytes(16), "While trying ", "ext", "via MP: ", "FixExt16 bytes")?;
			Some((sub_type, data))
		}
		_ => {
			None
		}
	})
}
