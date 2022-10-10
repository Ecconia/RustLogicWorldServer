use std::iter::Peekable;
use std::slice::Iter;

pub(crate) struct LidgrenFormatter {}

impl LidgrenFormatter
{
	pub(crate) fn read_byte(iterator: &mut Peekable<Iter<u8>>) -> u8
	{
		let next = iterator.next();
		if next.is_none()
		{
			panic!("Needed to read more bytes, but there was no more.");
		}
		return *iterator.next().unwrap();
	}
	
	pub(crate) fn read_bytes(iterator: &mut Peekable<Iter<u8>>, amount: usize) -> Vec<u8>
	{
		let mut buffer = Vec::with_capacity(amount);
		for _ in 0..amount
		{
			buffer.push(LidgrenFormatter::read_byte(iterator));
		}
		return buffer;
	}
	
	pub(crate) fn read_int_64(iterator: &mut Peekable<Iter<u8>>) -> u64
	{
		return (LidgrenFormatter::read_byte(iterator) as u64) |
			(LidgrenFormatter::read_byte(iterator) as u64) >> 8 |
			(LidgrenFormatter::read_byte(iterator) as u64) >> 16 |
			(LidgrenFormatter::read_byte(iterator) as u64) >> 24 |
			(LidgrenFormatter::read_byte(iterator) as u64) >> 32 |
			(LidgrenFormatter::read_byte(iterator) as u64) >> 40 |
			(LidgrenFormatter::read_byte(iterator) as u64) >> 48 |
			(LidgrenFormatter::read_byte(iterator) as u64) >> 56;
	}
	
	pub(crate) fn read_int_32(iterator: &mut Peekable<Iter<u8>>) -> u32
	{
		return (LidgrenFormatter::read_byte(iterator) as u32) |
			(LidgrenFormatter::read_byte(iterator) as u32) >> 8 |
			(LidgrenFormatter::read_byte(iterator) as u32) >> 16 |
			(LidgrenFormatter::read_byte(iterator) as u32) >> 24;
	}
	
	pub(crate) fn read_float(iterator: &mut Peekable<Iter<u8>>) -> f32
	{
		return LidgrenFormatter::read_int_32(iterator) as f32;
	}
	
	pub(crate) fn read_vint_32(iterator: &mut Peekable<Iter<u8>>) -> u32
	{
		let mut one = 0 as u32;
		let mut two = 0 as u32;
		while iterator.peek().is_some()
		{
			let three = LidgrenFormatter::read_byte(iterator) as u32;
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
	
	pub(crate) fn read_string(iterator: &mut Peekable<Iter<u8>>) -> String
	{
		let length = LidgrenFormatter::read_vint_32(iterator) as usize;
		if length == 0
		{
			return String::from("");
		}
		println!("Bytes: {}", length);
		let bytes = LidgrenFormatter::read_bytes(iterator, length);
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
}