use crate::custom_unwrap_result_or_else;

use crate::util::custom_iterator::CustomIterator;

pub fn read_int_64(iterator: &mut CustomIterator) -> u64
{
	if iterator.remaining() < 8 {
		panic!("Ran out of bytes, while reading int_64: {}/{}", iterator.remaining(), 8);
	}
	return (iterator.next_unchecked() as u64) |
		(iterator.next_unchecked() as u64) << 8 |
		(iterator.next_unchecked() as u64) << 16 |
		(iterator.next_unchecked() as u64) << 24 |
		(iterator.next_unchecked() as u64) << 32 |
		(iterator.next_unchecked() as u64) << 40 |
		(iterator.next_unchecked() as u64) << 48 |
		(iterator.next_unchecked() as u64) << 56;
}

pub fn write_int_64(buffer: &mut Vec<u8>, value: u64)
{
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

pub fn read_int_32(iterator: &mut CustomIterator) -> u32
{
	if iterator.remaining() < 4 {
		panic!("Ran out of bytes, while reading int_32: {}/{}", iterator.remaining(), 4);
	}
	return (iterator.next_unchecked() as u32) |
		(iterator.next_unchecked() as u32) >> 8 |
		(iterator.next_unchecked() as u32) >> 16 |
		(iterator.next_unchecked() as u32) >> 24;
}

pub fn write_int_32(buffer: &mut Vec<u8>, value: u32)
{
	let mut val = value;
	buffer.push(val as u8);
	val >>= 8;
	buffer.push(val as u8);
	val >>= 8;
	buffer.push(val as u8);
	val >>= 8;
	buffer.push(val as u8);
}

pub fn read_float(iterator: &mut CustomIterator) -> f32
{
	return read_int_32(iterator) as f32;
}

pub fn write_float(buffer: &mut Vec<u8>, value: f32)
{
	write_int_32(buffer, value as u32);
}

pub fn read_vint_32(iterator: &mut CustomIterator) -> Result<u32, String>
{
	let mut one = 0 as u32;
	let mut two = 0 as u32;
	loop
	{
		let three = custom_unwrap_result_or_else!(iterator.next(), (|message| {
			return Err(format!("While reading variable int, ran out of bytes:\n-> {}", message));
		})) as u32;
		one |= (three & 0x7f) << two;
		two += 7;
		if (three & 0x80) == 0
		{
			break;
		}
	}
	return Ok(one);
}

pub fn write_vint_32(buffer: &mut Vec<u8>, value: u32)
{
	let mut val = value;
	while val >= 0x80 {
		buffer.push((val as u8) | 0x80);
		val >>= 7;
	}
	buffer.push(val as u8);
}

pub fn read_string(iterator: &mut CustomIterator) -> Result<String, String>
{
	let length = custom_unwrap_result_or_else!(read_vint_32(iterator), (|message| {
		return Err(format!("While reading length of string, ran out of bytes:\n-> {}", message));
	})) as usize;
	if length == 0
	{
		return Ok(String::from(""));
	}
	let bytes = custom_unwrap_result_or_else!(iterator.read_bytes(length), (|message| {
		return Err(format!("While reading bytes of string, ran out of bytes:\n-> {}", message));
	}));
	Ok(custom_unwrap_result_or_else!(String::from_utf8(bytes), (|message| {
		return Err(format!("While constructing string from bytes, ran into encoding error:\n-> {}", message));
	})))
}

pub fn write_string(buffer: &mut Vec<u8>, value: &str)
{
	let bytes = value.as_bytes();
	write_vint_32(buffer, bytes.len() as u32);
	buffer.extend(bytes.iter());
}
