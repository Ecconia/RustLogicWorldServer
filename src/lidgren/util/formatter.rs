use crate::prelude::*;

use crate::util::custom_iterator::CustomIterator;

pub fn read_int_64(iterator: &mut CustomIterator) -> EhResult<u64> {
	if iterator.remaining() < 8 {
		return exception!("Ran out of bytes, while reading int_64: ", iterator.remaining(), "/", 8);
	}
	Ok((iterator.next_unchecked() as u64) |
		(iterator.next_unchecked() as u64) << 8 |
		(iterator.next_unchecked() as u64) << 16 |
		(iterator.next_unchecked() as u64) << 24 |
		(iterator.next_unchecked() as u64) << 32 |
		(iterator.next_unchecked() as u64) << 40 |
		(iterator.next_unchecked() as u64) << 48 |
		(iterator.next_unchecked() as u64) << 56)
}

pub fn write_int_64(buffer: &mut Vec<u8>, value: u64) {
	let mut val = value;
	buffer.push(val as u8);
	val >>= 8;
	buffer.push(val as u8);
	val >>= 8;
	buffer.push(val as u8);
	val >>= 8;
	buffer.push(val as u8);
	val >>= 8;
	buffer.push(val as u8);
	val >>= 8;
	buffer.push(val as u8);
	val >>= 8;
	buffer.push(val as u8);
	val >>= 8;
	buffer.push(val as u8);
}

pub fn read_int_32(iterator: &mut CustomIterator) -> EhResult<u32> {
	if iterator.remaining() < 4 {
		return exception!("Ran out of bytes, while reading int_32: ", iterator.remaining(), "/", 4);
	}
	Ok((iterator.next_unchecked() as u32) |
		(iterator.next_unchecked() as u32) >> 8 |
		(iterator.next_unchecked() as u32) >> 16 |
		(iterator.next_unchecked() as u32) >> 24)
}

pub fn write_int_32(buffer: &mut Vec<u8>, value: u32) {
	let mut val = value;
	buffer.push(val as u8);
	val >>= 8;
	buffer.push(val as u8);
	val >>= 8;
	buffer.push(val as u8);
	val >>= 8;
	buffer.push(val as u8);
}

pub fn read_float(iterator: &mut CustomIterator) -> EhResult<f32> {
	Ok(exception_wrap!(read_int_32(iterator), "While converting int to float")? as f32)
}

pub fn write_float(buffer: &mut Vec<u8>, value: f32) {
	write_int_32(buffer, value as u32);
}

pub fn read_vint_32(iterator: &mut CustomIterator) -> EhResult<u32> {
	let mut one = 0_u32;
	let mut two = 0_u32;
	loop {
		let three = exception_wrap!(iterator.next(), "While reading variable int")? as u32;
		one |= (three & 0x7f) << two;
		two += 7;
		if (three & 0x80) == 0 {
			break;
		}
	}
	Ok(one)
}

pub fn write_vint_32(buffer: &mut Vec<u8>, value: u32) {
	let mut val = value;
	while val >= 0x80 {
		buffer.push((val as u8) | 0x80);
		val >>= 7;
	}
	buffer.push(val as u8);
}

pub fn read_string(iterator: &mut CustomIterator) -> EhResult<String> {
	let length = exception_wrap!(read_vint_32(iterator), "While reading length of string")? as usize;
	if length == 0 {
		return Ok(String::from(""));
	}
	let bytes = exception_wrap!(iterator.read_bytes(length), "While reading bytes of string")?;
	exception_from!(String::from_utf8(bytes), "While converting string from bytes")
}

pub fn write_string(buffer: &mut Vec<u8>, value: &str) {
	let bytes = value.as_bytes();
	write_vint_32(buffer, bytes.len() as u32);
	buffer.extend(bytes.iter());
}
