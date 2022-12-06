use crate::error_handling::ResultErrorExt;

use crate::util::custom_iterator::CustomIterator;

pub fn read_int_64(iterator: &mut CustomIterator) -> Result<u64, String> {
	if iterator.remaining() < 8 {
		return Err(format!("Ran out of bytes, while reading int_64: {}/{}", iterator.remaining(), 8));
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

pub fn read_int_32(iterator: &mut CustomIterator) -> Result<u32, String> {
	if iterator.remaining() < 4 {
		return Err(format!("Ran out of bytes, while reading int_32: {}/{}", iterator.remaining(), 4));
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

pub fn read_float(iterator: &mut CustomIterator) -> Result<f32, String> {
	return Ok(read_int_32(iterator).forward_error("While converting int to float")? as f32);
}

pub fn write_float(buffer: &mut Vec<u8>, value: f32) {
	write_int_32(buffer, value as u32);
}

pub fn read_vint_32(iterator: &mut CustomIterator) -> Result<u32, String> {
	let mut one = 0 as u32;
	let mut two = 0 as u32;
	loop {
		let three = iterator.next().forward_error("While reading variable int, ran out of bytes:")? as u32;
		one |= (three & 0x7f) << two;
		two += 7;
		if (three & 0x80) == 0 {
			break;
		}
	}
	return Ok(one);
}

pub fn write_vint_32(buffer: &mut Vec<u8>, value: u32) {
	let mut val = value;
	while val >= 0x80 {
		buffer.push((val as u8) | 0x80);
		val >>= 7;
	}
	buffer.push(val as u8);
}

pub fn read_string(iterator: &mut CustomIterator) -> Result<String, String> {
	let length = read_vint_32(iterator).forward_error("While reading length of string, ran out of bytes:")? as usize;
	if length == 0 {
		return Ok(String::from(""));
	}
	let bytes = iterator.read_bytes(length).forward_error("While reading bytes of string, ran out of bytes:")?;
	Ok(String::from_utf8(bytes).forward_error("While constructing string from bytes, ran into encoding error")?)
}

pub fn write_string(buffer: &mut Vec<u8>, value: &str) {
	let bytes = value.as_bytes();
	write_vint_32(buffer, bytes.len() as u32);
	buffer.extend(bytes.iter());
}
