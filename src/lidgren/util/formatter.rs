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
	Ok(read_int_32(iterator).wrap(ex!("While converting int to float"))? as f32)
}

pub fn write_float(buffer: &mut Vec<u8>, value: f32) {
	write_int_32(buffer, value as u32);
}

pub fn read_vint_32(iterator: &mut CustomIterator) -> EhResult<u32> {
	let mut one = 0_u32;
	let mut two = 0_u32;
	loop {
		let three = iterator.next().wrap(ex!("While reading variable int"))? as u32;
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

pub fn vint_length(value: u32) -> u32 {
	match value {
		0..=0b00000000000000000000000010000000 => 1,
		0..=0b00000000000000000100000000000000 => 2,
		0..=0b00000000001000000000000000000000 => 3,
		0..=0b00010000000000000000000000000000 => 4,
		_ => 5,
	}
}

pub fn read_string(iterator: &mut CustomIterator) -> EhResult<String> {
	let length = read_vint_32(iterator).wrap(ex!("While reading length of string"))? as usize;
	if length == 0 {
		return Ok(String::from(""));
	}
	let bytes = iterator.read_bytes(length).wrap(ex!("While reading bytes of string"))?;
	String::from_utf8(bytes).map_ex(ex!("While converting string from bytes"))
}

pub fn write_string(buffer: &mut Vec<u8>, value: &str) {
	let bytes = value.as_bytes();
	write_vint_32(buffer, bytes.len() as u32);
	buffer.extend(bytes.iter());
}
