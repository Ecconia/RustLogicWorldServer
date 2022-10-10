use std::iter::Peekable;
use std::slice::Iter;

pub fn read_byte(iterator: &mut Peekable<Iter<u8>>) -> u8
{
	let next = iterator.next();
	if next.is_none()
	{
		panic!("Needed to read more bytes, but there was no more.");
	}
	return *iterator.next().unwrap();
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
		(read_byte(iterator) as u64) >> 8 |
		(read_byte(iterator) as u64) >> 16 |
		(read_byte(iterator) as u64) >> 24 |
		(read_byte(iterator) as u64) >> 32 |
		(read_byte(iterator) as u64) >> 40 |
		(read_byte(iterator) as u64) >> 48 |
		(read_byte(iterator) as u64) >> 56;
}

pub fn read_int_32(iterator: &mut Peekable<Iter<u8>>) -> u32
{
	return (read_byte(iterator) as u32) |
		(read_byte(iterator) as u32) >> 8 |
		(read_byte(iterator) as u32) >> 16 |
		(read_byte(iterator) as u32) >> 24;
}

pub fn read_float(iterator: &mut Peekable<Iter<u8>>) -> f32
{
	return read_int_32(iterator) as f32;
}

pub fn read_vint_32(iterator: &mut Peekable<Iter<u8>>) -> u32
{
	let mut one = 0 as u32;
	let mut two = 0 as u32;
	while iterator.peek().is_some()
	{
		let three = read_byte(iterator) as u32;
		println!("Read byte: {}", three);
		one |= (three & 0x7f) << two;
		two += 7;
		if (three & 0x80) == 0
		{
			println!("Break!");
			break;
		}
	}
	return one;
}

pub fn read_string(iterator: &mut Peekable<Iter<u8>>) -> String
{
	let length = read_vint_32(iterator) as usize;
	if length == 0
	{
		return String::from("");
	}
	println!("Bytes: {}", length);
	let bytes = read_bytes(iterator, length);
	// print!("bytes:");
	// for b in bytes
	// {
	// 	print!(" {}", b);
	// }
	println!("Bytes: {:x?}", bytes);
	let text = String::from_utf8(bytes);
	if text.is_err()
	{
		panic!("Could not create string from bytes: {}", text.err().unwrap())
	}
	return text.unwrap();
}
