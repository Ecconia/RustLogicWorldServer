use crate::error_handling::custom_unwrap_result_or_else;

use crate::util::custom_iterator::CustomIterator;

//Integers:

pub fn read_int_8(iterator: &mut CustomIterator) -> u8 {
	return custom_unwrap_result_or_else!(iterator.next(), (|message| {
		panic!("Could not read MP u8, because ran out of bytes: {}", message);
	}));
}

pub fn read_int_16(iterator: &mut CustomIterator) -> u16 {
	if iterator.remaining() < 2 {
		panic!("Could not read MP u16, because ran out of bytes.");
	}
	return (iterator.next_unchecked() as u16) << 8 | iterator.next_unchecked() as u16;
}

pub fn read_int_32(iterator: &mut CustomIterator) -> u32 {
	if iterator.remaining() < 4 {
		panic!("Could not read MP u32, because ran out of bytes.");
	}
	return (iterator.next_unchecked() as u32) << 24 |
		(iterator.next_unchecked() as u32) << 16 |
		(iterator.next_unchecked() as u32) << 8 |
		(iterator.next_unchecked() as u32);
}

pub fn read_int_auto(iterator: &mut CustomIterator) -> u32 {
	let type_fml = custom_unwrap_result_or_else!(iterator.next(), (|message| {
		panic!("Could not read MP unsigned int, because ran out of bytes: {}", message);
	}));
	match type_fml {
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

pub fn read_map_flex(iterator: &mut CustomIterator) -> u32 {
	let next = custom_unwrap_result_or_else!(iterator.next(), (|message| {
		panic!("Could not read MP flex map, because ran out of bytes: {}", message);
	}));
	return (next as u32) - 0x80;
}

pub fn read_map_auto(iterator: &mut CustomIterator) -> u32 {
	let type_fml = custom_unwrap_result_or_else!(iterator.peek(), (|message| {
		panic!("Could not read MP map, because ran out of bytes: {}", message);
	}));
	match type_fml {
		0x80..=0x91 => {
			read_map_flex(iterator) as u32
		}
		0xDE => {
			iterator.skip();
			read_int_16(iterator) as u32
		}
		0xDF => {
			iterator.skip();
			read_int_32(iterator) as u32
		}
		_ => {
			panic!("Expected map, but got type code {:?}", type_fml);
		}
	}
}

//Array:

pub fn read_array_flex(iterator: &mut CustomIterator) -> u32 {
	let next = custom_unwrap_result_or_else!(iterator.next(), (|message| {
		panic!("Could not read MP flex array, because ran out of bytes: {}", message);
	}));
	return (next as u32) - 0x90;
}

pub fn read_array_auto(iterator: &mut CustomIterator) -> u32 {
	let type_fml = custom_unwrap_result_or_else!(iterator.peek(), (|message| {
		panic!("Could not read MP array, because ran out of bytes: {}", message);
	}));
	match type_fml {
		0x90..=0xA1 => {
			read_array_flex(iterator) as u32
		}
		0xDC => {
			iterator.skip();
			read_int_16(iterator) as u32
		}
		0xDD => {
			iterator.skip();
			read_int_32(iterator) as u32
		}
		_ => {
			panic!("Expected array, but got type code {:?}", type_fml);
		}
	}
}

//String:

fn read_string_len(iterator: &mut CustomIterator, length: usize) -> String {
	let bytes = custom_unwrap_result_or_else!(iterator.read_bytes(length), (|message| {
		panic!("Could not read MP string bytes, because ran out of bytes: {}", message);
	}));
	return String::from_utf8(bytes).unwrap();
}

pub fn read_string_flex(iterator: &mut CustomIterator) -> String {
	let next = custom_unwrap_result_or_else!(iterator.next(), (|message| {
		panic!("Could not read MP flex string, because ran out of bytes: {}", message);
	}));
	let length = ((next as u32) - 0xA0) as usize;
	return read_string_len(iterator, length);
}

pub fn read_string_8(iterator: &mut CustomIterator) -> String {
	let length = read_int_8(iterator) as usize;
	return read_string_len(iterator, length);
}

pub fn read_string_16(iterator: &mut CustomIterator) -> String {
	let length = read_int_16(iterator) as usize;
	return read_string_len(iterator, length);
}

pub fn read_string_auto(iterator: &mut CustomIterator) -> Option<String> {
	let type_fml = custom_unwrap_result_or_else!(iterator.peek(), (|message| {
		panic!("Could not read MP string, because ran out of bytes: {}", message);
	}));
	match type_fml {
		0xA0..=0xBF => {
			Some(read_string_flex(iterator))
		}
		0xC0 => {
			iterator.skip();
			None
		}
		0xD9 => {
			iterator.skip();
			Some(read_string_8(iterator))
		}
		0xDA => {
			iterator.skip();
			Some(read_string_16(iterator))
		}
		_ => {
			panic!("Expected string, but got type code {:?}", type_fml);
		}
	}
}

//Boolean:

pub fn read_bool_auto(iterator: &mut CustomIterator) -> bool {
	let type_fml = custom_unwrap_result_or_else!(iterator.next(), (|message| {
		panic!("Could not read MP bool, because ran out of bytes: {}", message);
	}));
	match type_fml {
		0xC2 => false,
		0xC3 => true,
		_ => panic!("Expected boolean, but got type code {:?}", type_fml)
	}
}

//Binary:

pub fn read_binary_len(iterator: &mut CustomIterator, length: usize) -> Vec<u8> {
	return custom_unwrap_result_or_else!(iterator.read_bytes(length), (|message| {
		panic!("Could not read MP binary bytes, because ran out of bytes: {}", message)
	}));
}

pub fn read_binary_8(iterator: &mut CustomIterator) -> Vec<u8> {
	let length = read_int_8(iterator) as usize;
	return read_binary_len(iterator, length);
}

pub fn read_binary_16(iterator: &mut CustomIterator) -> Vec<u8> {
	let length = read_int_16(iterator) as usize;
	return read_binary_len(iterator, length);
}

pub fn read_binary_32(iterator: &mut CustomIterator) -> Vec<u8> {
	let length = read_int_32(iterator) as usize;
	return read_binary_len(iterator, length);
}

pub fn read_binary_auto(iterator: &mut CustomIterator) -> Option<Vec<u8>> {
	let type_fml = custom_unwrap_result_or_else!(iterator.next(), (|message| {
		panic!("Could not read MP binary, because ran out of bytes: {}", message);
	}));
	match type_fml {
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
