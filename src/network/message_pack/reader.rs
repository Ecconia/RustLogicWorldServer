use crate::error_handling::ResultErrorExt;
use crate::util::custom_iterator::CustomIterator;

//Integers:

pub fn read_int_8(iterator: &mut CustomIterator) -> Result<u8, String> {
	iterator.next().forward_error("Could not read MP u8, because ran out of bytes")
}

pub fn read_int_16(iterator: &mut CustomIterator) -> Result<u16, String> {
	if iterator.remaining() < 2 {
		return Err("Could not read MP u16, because ran out of bytes.".to_string());
	}
	Ok((iterator.next_unchecked() as u16) << 8 | iterator.next_unchecked() as u16)
}

pub fn read_int_32(iterator: &mut CustomIterator) -> Result<u32, String> {
	if iterator.remaining() < 4 {
		return Err("Could not read MP u32, because ran out of bytes.".to_string());
	}
	Ok((iterator.next_unchecked() as u32) << 24 |
		(iterator.next_unchecked() as u32) << 16 |
		(iterator.next_unchecked() as u32) << 8 |
		(iterator.next_unchecked() as u32))
}

pub fn read_int_auto(iterator: &mut CustomIterator) -> Result<u32, String> {
	let type_fml = iterator.next().forward_error("Could not read MP unsigned int, because ran out of bytes:")?;
	match type_fml {
		0..=0x80 => {
			Ok(type_fml as u32)
		}
		0xCC => {
			Ok(read_int_8(iterator).forward_error("While automatically reading a 8 bit integer")? as u32)
		}
		0xCD => {
			Ok(read_int_16(iterator).forward_error("While automatically reading a 16 bit integer")? as u32)
		}
		0xCE => {
			Ok(read_int_32(iterator).forward_error("While automatically reading a 32 bit integer")? as u32)
		}
		_ => {
			Err(format!("Expected integer, but got type code {:?}", type_fml))
		}
	}
}

//Map:

pub fn read_map_flex(iterator: &mut CustomIterator) -> Result<u32, String> {
	let next = iterator.next().forward_error("Could not read MP flex map, because ran out of bytes")?;
	Ok((next as u32) - 0x80)
}

pub fn read_map_auto(iterator: &mut CustomIterator) -> Result<u32, String> {
	let type_fml = iterator.peek().forward_error("Could not read MP map, because ran out of bytes")?;
	match type_fml {
		0x80..=0x91 => {
			Ok(read_map_flex(iterator).forward_error("While automatically reading a flex map")? as u32)
		}
		0xDE => {
			iterator.skip();
			Ok(read_int_16(iterator).forward_error("While automatically reading a 16 bit map")? as u32)
		}
		0xDF => {
			iterator.skip();
			Ok(read_int_32(iterator).forward_error("While automatically reading a 32 bit map")? as u32)
		}
		_ => {
			Err(format!("Expected map, but got type code {:?}", type_fml))
		}
	}
}

//Array:

pub fn read_array_flex(iterator: &mut CustomIterator) -> Result<u32, String> {
	let next = iterator.next().forward_error("Could not read MP flex array, because ran out of bytes")?;
	Ok((next as u32) - 0x90)
}

pub fn read_array_auto(iterator: &mut CustomIterator) -> Result<u32, String> {
	let type_fml = iterator.peek().forward_error("Could not read MP array, because ran out of bytes")?;
	match type_fml {
		0x90..=0xA1 => {
			Ok(read_array_flex(iterator).forward_error("While automatically reading a flex array")? as u32)
		}
		0xDC => {
			iterator.skip();
			Ok(read_int_16(iterator).forward_error("While automatically reading a 16 bit array")? as u32)
		}
		0xDD => {
			iterator.skip();
			Ok(read_int_32(iterator).forward_error("While automatically reading a 32 bit array")? as u32)
		}
		_ => {
			Err(format!("Expected array, but got type code {:?}", type_fml))
		}
	}
}

//String:

fn read_string_len(iterator: &mut CustomIterator, length: usize) -> Result<String, String> {
	let bytes = iterator.read_bytes(length).forward_error("Could not read MP string bytes, because ran out of bytes")?;
	String::from_utf8(bytes).forward_error("While converting bytes to fixed length string")
}

pub fn read_string_flex(iterator: &mut CustomIterator) -> Result<String, String> {
	let next = iterator.next().forward_error("Could not read MP flex string, because ran out of bytes")?;
	let length = ((next as u32) - 0xA0) as usize;
	read_string_len(iterator, length).forward_error("While reading a flex string")
}

pub fn read_string_8(iterator: &mut CustomIterator) -> Result<String, String> {
	let length = read_int_8(iterator).forward_error("While reading an 8 bit string prefix")? as usize;
	read_string_len(iterator, length)
}

pub fn read_string_16(iterator: &mut CustomIterator) -> Result<String, String> {
	let length = read_int_16(iterator).forward_error("While reading a 16 bit string prefix")? as usize;
	read_string_len(iterator, length)
}

pub fn read_string_auto(iterator: &mut CustomIterator) -> Result<Option<String>, String> {
	let type_fml = iterator.peek().forward_error("Could not read MP string, because ran out of bytes")?;
	match type_fml {
		0xA0..=0xBF => {
			Ok(Some(read_string_flex(iterator).forward_error("While automatically reading a flex string")?))
		}
		0xC0 => {
			iterator.skip();
			Ok(None)
		}
		0xD9 => {
			iterator.skip();
			Ok(Some(read_string_8(iterator).forward_error("While automatically reading a 8 bit prefixed length string")?))
		}
		0xDA => {
			iterator.skip();
			Ok(Some(read_string_16(iterator).forward_error("While automatically reading a 16 bit prefixed length string")?))
		}
		_ => {
			Err(format!("Expected string, but got type code {:?}", type_fml))
		}
	}
}

//Boolean:

pub fn read_bool_auto(iterator: &mut CustomIterator) -> Result<bool, String> {
	let type_fml = iterator.next().forward_error("Could not read MP bool, because ran out of bytes")?;
	match type_fml {
		0xC2 => Ok(false),
		0xC3 => Ok(true),
		_ => Err(format!("Expected boolean, but got type code {:?}", type_fml))
	}
}

//Binary:

pub fn read_binary_len(iterator: &mut CustomIterator, length: usize) -> Result<Vec<u8>, String> {
	iterator.read_bytes(length).forward_error("Could not read MP binary bytes, because ran out of bytes")
}

pub fn read_binary_8(iterator: &mut CustomIterator) -> Result<Vec<u8>, String> {
	let length = read_int_8(iterator).forward_error("While reading an 8 bit binary length prefix")? as usize;
	read_binary_len(iterator, length)
}

pub fn read_binary_16(iterator: &mut CustomIterator) -> Result<Vec<u8>, String> {
	let length = read_int_16(iterator).forward_error("While reading a 16 bit binary length prefix")? as usize;
	read_binary_len(iterator, length)
}

pub fn read_binary_32(iterator: &mut CustomIterator) -> Result<Vec<u8>, String> {
	let length = read_int_32(iterator).forward_error("While reading a 32 bit binary length prefix")? as usize;
	read_binary_len(iterator, length)
}

pub fn read_binary_auto(iterator: &mut CustomIterator) -> Result<Option<Vec<u8>>, String> {
	let type_fml = iterator.next().forward_error("Could not read MP binary, because ran out of bytes")?;
	match type_fml {
		0xC0 => {
			Ok(None)
		}
		0xC4 => {
			Ok(Some(read_binary_8(iterator).forward_error("While automatically reading an 8 bit binary section")?))
		}
		0xC5 => {
			Ok(Some(read_binary_16(iterator).forward_error("While automatically reading a 16 bit binary section")?))
		}
		0xC6 => {
			Ok(Some(read_binary_32(iterator).forward_error("While automatically reading a 32 bit binary section")?))
		}
		_ => {
			Err(format!("Expected byte array, but got type code {:?}", type_fml))
		}
	}
}
