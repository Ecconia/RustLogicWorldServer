use std::iter::Peekable;
use std::slice::Iter;

//Integers:

pub fn read_int_8(iterator: &mut Peekable<Iter<u8>>) -> u8
{
	return *iterator.next().unwrap();
}

pub fn read_int_16(iterator: &mut Peekable<Iter<u8>>) -> u16
{
	return (*iterator.next().unwrap() as u16) << 8 | *iterator.next().unwrap() as u16;
}

pub fn read_int_32(iterator: &mut Peekable<Iter<u8>>) -> u32
{
	return (*iterator.next().unwrap() as u32) << 24 |
		(*iterator.next().unwrap() as u32) << 16 |
		(*iterator.next().unwrap() as u32) << 8 |
		(*iterator.next().unwrap() as u32);
}

pub fn read_int_auto(iterator: &mut Peekable<Iter<u8>>) -> u32
{
	let type_fml = *iterator.next().unwrap();
	match type_fml
	{
		0..=0x80 => {
			type_fml as u32
		}
		0xCC => {
			read_int_8(iterator) as u32
		}
		0xCD => {
			read_int_16(iterator) as u32
		}
		0xCE => {
			read_int_32(iterator) as u32
		}
		_ => {
			panic!("Expected integer, but got type code {:?}", type_fml);
		}
	}
}

//Map:

pub fn read_map_flex(iterator: &mut Peekable<Iter<u8>>) -> u32
{
	return (*iterator.next().unwrap() as u32) - 0x80;
}

pub fn read_map_auto(iterator: &mut Peekable<Iter<u8>>) -> u32
{
	let type_fml = **iterator.peek().unwrap();
	match type_fml
	{
		0x80..=0x91 => {
			read_map_flex(iterator) as u32
		}
		0xDE => {
			iterator.next();
			read_int_16(iterator) as u32
		}
		0xDF => {
			iterator.next();
			read_int_32(iterator) as u32
		}
		_ => {
			panic!("Expected map, but got type code {:?}", type_fml);
		}
	}
}

//Array:

pub fn read_array_flex(iterator: &mut Peekable<Iter<u8>>) -> u32
{
	return (*iterator.next().unwrap() as u32) - 0x90;
}

pub fn read_array_auto(iterator: &mut Peekable<Iter<u8>>) -> u32
{
	let type_fml = **iterator.peek().unwrap();
	match type_fml
	{
		0x90..=0xA1 => {
			read_array_flex(iterator) as u32
		}
		0xDC => {
			iterator.next();
			read_int_16(iterator) as u32
		}
		0xDD => {
			iterator.next();
			read_int_32(iterator) as u32
		}
		_ => {
			panic!("Expected array, but got type code {:?}", type_fml);
		}
	}
}

//String:

fn read_string_len(iterator: &mut Peekable<Iter<u8>>, length: usize) -> String
{
	let mut buffer = Vec::with_capacity(length);
	for _ in 0..length
	{
		buffer.push(*iterator.next().unwrap());
	}
	return String::from_utf8(buffer).unwrap();
}

pub fn read_string_flex(iterator: &mut Peekable<Iter<u8>>) -> String
{
	let length = ((*iterator.next().unwrap() as u32) - 0xA0) as usize;
	return read_string_len(iterator, length);
}

pub fn read_string_8(iterator: &mut Peekable<Iter<u8>>) -> String
{
	let length = read_int_8(iterator) as usize;
	return read_string_len(iterator, length);
}

pub fn read_string_16(iterator: &mut Peekable<Iter<u8>>) -> String
{
	let length = read_int_16(iterator) as usize;
	return read_string_len(iterator, length);
}

pub fn read_string_auto(iterator: &mut Peekable<Iter<u8>>) -> Option<String>
{
	let type_fml = **iterator.peek().unwrap();
	match type_fml
	{
		0xA0..=0xBF => {
			Some(read_string_flex(iterator))
		}
		0xC0 => {
			iterator.next();
			None
		}
		0xD9 => {
			iterator.next();
			Some(read_string_8(iterator))
		}
		0xDA => {
			iterator.next();
			Some(read_string_16(iterator))
		}
		_ => {
			panic!("Expected string, but got type code {:?}", type_fml);
		}
	}
}

//Boolean:

pub fn read_bool_auto(iterator: &mut Peekable<Iter<u8>>) -> bool
{
	let type_fml = *iterator.next().unwrap();
	match type_fml {
		0xC2 => false,
		0xC3 => true,
		_ => panic!("Expected boolean, but got type code {:?}", type_fml)
	}
}

//Binary:

pub fn read_binary_len(iterator: &mut Peekable<Iter<u8>>, length: usize) -> Vec<u8>
{
	let mut buffer = Vec::with_capacity(length);
	for _ in 0..length
	{
		buffer.push(*iterator.next().unwrap());
	}
	return buffer;
}

pub fn read_binary_8(iterator: &mut Peekable<Iter<u8>>) -> Vec<u8>
{
	let length = read_int_8(iterator) as usize;
	return read_binary_len(iterator, length);
}

pub fn read_binary_16(iterator: &mut Peekable<Iter<u8>>) -> Vec<u8>
{
	let length = read_int_16(iterator) as usize;
	return read_binary_len(iterator, length);
}

pub fn read_binary_32(iterator: &mut Peekable<Iter<u8>>) -> Vec<u8>
{
	let length = read_int_32(iterator) as usize;
	return read_binary_len(iterator, length);
}

pub fn read_binary_auto(iterator: &mut Peekable<Iter<u8>>) -> Option<Vec<u8>>
{
	let type_fml = *iterator.next().unwrap();
	match type_fml
	{
		0xC0 => {
			None
		}
		0xC4 => {
			Some(read_binary_8(iterator))
		}
		0xC5 => {
			Some(read_binary_16(iterator))
		}
		0xC6 => {
			Some(read_binary_32(iterator))
		}
		_ => {
			panic!("Expected byte array, but got type code {:?}", type_fml);
		}
	}
}
