//Misc:

pub fn write_null(buffer: &mut Vec<u8>)
{
	buffer.push(0xC0);
}

//Integers:

pub fn write_int_8(buffer: &mut Vec<u8>, value: u8)
{
	buffer.push(value);
}

pub fn write_int_16(buffer: &mut Vec<u8>, value: u16)
{
	buffer.push(0xCD);
	buffer.push((value >> 8) as u8);
	buffer.push(value as u8);
}

pub fn write_int_32(buffer: &mut Vec<u8>, value: u32)
{
	buffer.push(0xCE);
	buffer.push((value >> 24) as u8);
	buffer.push((value >> 16) as u8);
	buffer.push((value >> 8) as u8);
	buffer.push(value as u8);
}

pub fn write_int_auto(buffer: &mut Vec<u8>, value: u32)
{
	match value
	{
		0..=0x7F => buffer.push(value as u8),
		0..=0xFF => write_int_8(buffer, value as u8),
		0..=0xFFFF => write_int_16(buffer, value as u16),
		_ => write_int_32(buffer, value)
	};
}

//Strings:

pub fn write_string_flex(buffer: &mut Vec<u8>, value: String)
{
	let bytes = value.as_bytes();
	let length = bytes.len();
	if length > 31
	{
		panic!("Attempted to write a string of length {} with flex type, but only 31 characters are possible.", length);
	}
	buffer.push(0b10100000 | length as u8);
	buffer.extend(bytes.iter());
}

pub fn write_string_8(buffer: &mut Vec<u8>, value: String)
{
	let bytes = value.as_bytes();
	let length = bytes.len();
	if length > 0xFF
	{
		panic!("Attempted to write a string of length {} with flex type, but only 0xFF characters are possible.", length);
	}
	buffer.push(0xD9);
	buffer.push(length as u8);
	buffer.extend(bytes.iter());
}

pub fn write_string_16(buffer: &mut Vec<u8>, value: String)
{
	let bytes = value.as_bytes();
	let length = bytes.len();
	if length > 0xFFFF
	{
		panic!("Attempted to write a string of length {} with flex type, but only 0xFFFF characters are possible.", length);
	}
	buffer.push(0xDA);
	buffer.push((length >> 8) as u8);
	buffer.push(length as u8);
	buffer.extend(bytes.iter());
}

pub fn write_string_auto(buffer: &mut Vec<u8>, value: Option<String>)
{
	if value.is_none()
	{
		write_null(buffer);
		return;
	}
	
	let text = value.unwrap();
	let bytes = text.as_bytes();
	let length = bytes.len();
	match length
	{
		0..=31 => write_string_flex(buffer, text),
		0..=0xFF => write_string_8(buffer, text),
		0..=0xFFFF => write_string_16(buffer, text),
		_ => {
			panic!("String to write is too large: {}", length)
		}
	}
}

//Booleans:

pub fn write_bool(buffer: &mut Vec<u8>, value: bool)
{
	if value {
		buffer.push(0xC3);
	} else {
		buffer.push(0xC2);
	}
}

//Map:

pub fn write_map_flex(buffer: &mut Vec<u8>, value: u32)
{
	if value > 15
	{
		panic!("Flex maps only support up to 15 entries. Provided {}", value);
	}
	buffer.push(0x80 + value as u8);
}

pub fn write_map_auto(buffer: &mut Vec<u8>, value: u32)
{
	match value
	{
		0..=0xF => write_map_flex(buffer, value),
		_ => panic!("Not implemented yet.")
	}
}
