use std::iter::Peekable;
use std::slice::Iter;

pub fn read_byte(iterator: &mut Peekable<Iter<u8>>) -> u8
{
	let next = iterator.next();
	if next.is_none()
	{
		panic!("Needed to read more bytes, but there was no more.");
	}
	return *next.unwrap();
}

pub fn read_bytes(iterator: &mut Peekable<Iter<u8>>, amount: usize) -> Vec<u8>
{
	let mut buffer = Vec::with_capacity(amount);
	for _ in 0..amount
	{
		buffer.push(read_byte(iterator));
	}
	return buffer;
}

pub fn read_int_64(iterator: &mut Peekable<Iter<u8>>) -> u64
{
	return (read_byte(iterator) as u64) |
		(read_byte(iterator) as u64) << 8 |
		(read_byte(iterator) as u64) << 16 |
		(read_byte(iterator) as u64) << 24 |
		(read_byte(iterator) as u64) << 32 |
		(read_byte(iterator) as u64) << 40 |
		(read_byte(iterator) as u64) << 48 |
		(read_byte(iterator) as u64) << 56;
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

pub fn read_int_32(iterator: &mut Peekable<Iter<u8>>) -> u32
{
	return (read_byte(iterator) as u32) |
		(read_byte(iterator) as u32) >> 8 |
		(read_byte(iterator) as u32) >> 16 |
		(read_byte(iterator) as u32) >> 24;
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

pub fn read_float(iterator: &mut Peekable<Iter<u8>>) -> f32
{
	return read_int_32(iterator) as f32;
}

pub fn write_float(buffer: &mut Vec<u8>, value: f32)
{
	write_int_32(buffer, value as u32);
}

pub fn read_vint_32(iterator: &mut Peekable<Iter<u8>>) -> u32
{
	let mut one = 0 as u32;
	let mut two = 0 as u32;
	while iterator.peek().is_some()
	{
		let three = read_byte(iterator) as u32;
		one |= (three & 0x7f) << two;
		two += 7;
		if (three & 0x80) == 0
		{
			break;
		}
	}
	return one;
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

pub fn read_string(iterator: &mut Peekable<Iter<u8>>) -> String
{
	let length = read_vint_32(iterator) as usize;
	if length == 0
	{
		return String::from("");
	}
	let bytes = read_bytes(iterator, length);
	let text = String::from_utf8(bytes);
	if text.is_err()
	{
		panic!("Could not create string from bytes: {}", text.err().unwrap())
	}
	return text.unwrap();
}

pub fn write_string(buffer: &mut Vec<u8>, value: &str)
{
	let bytes = value.as_bytes();
	write_vint_32(buffer, bytes.len() as u32);
	buffer.extend(bytes.iter());
}
