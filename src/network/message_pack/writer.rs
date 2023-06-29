//Misc:

pub fn write_null(buffer: &mut Vec<u8>) {
	buffer.push(0xC0);
}

//Integers:

pub fn write_int_8(buffer: &mut Vec<u8>, value: u8) {
	buffer.push(0xCC);
	buffer.push(value);
}

pub fn write_int_16(buffer: &mut Vec<u8>, value: u16) {
	buffer.push(0xCD);
	buffer.push((value >> 8) as u8);
	buffer.push(value as u8);
}

pub fn write_int_32(buffer: &mut Vec<u8>, value: u32) {
	buffer.push(0xCE);
	buffer.push((value >> 24) as u8);
	buffer.push((value >> 16) as u8);
	buffer.push((value >> 8) as u8);
	buffer.push(value as u8);
}

pub fn write_int_auto(buffer: &mut Vec<u8>, value: u32) {
	match value {
		0..=0x7F => buffer.push(value as u8),
		0x80..=0xFF => write_int_8(buffer, value as u8),
		0x100..=0xFFFF => write_int_16(buffer, value as u16),
		0x10000..=0xFFFFFFFF => write_int_32(buffer, value),
	};
}

pub fn write_i32(buffer: &mut Vec<u8>, value: i32) {
	//TODO: Use match statement, as soon as 'const{}' is available.
	if value > 0 {
		write_int_auto(buffer, value as u32);
	} else if value >= -32 {
		buffer.push(value as u8);
	} else if value >= i8::MIN as i32 {
		buffer.push(0xD0);
		buffer.push(value as u8);
	} else if value >= i16::MIN as i32 {
		buffer.push(0xD1);
		buffer.push((value >> 8) as u8);
		buffer.push(value as u8);
	} else if value >= i32::MIN {
		buffer.push(0xD2);
		buffer.push((value >> 24) as u8);
		buffer.push((value >> 16) as u8);
		buffer.push((value >> 8) as u8);
		buffer.push(value as u8);
	} else {
		panic!("It is impossible to reach this point in code, unless the developer messed up or someone copied this without care.");
	}
}

//Floats:

pub fn write_float_auto(buffer: &mut Vec<u8>, value: f32) {
	buffer.push(0xCA);
	let float_as_bits = value.to_bits();
	buffer.push((float_as_bits >> 24) as u8);
	buffer.push((float_as_bits >> 16) as u8);
	buffer.push((float_as_bits >> 8) as u8);
	buffer.push(float_as_bits as u8);
}

//Strings:

pub fn write_string_flex(buffer: &mut Vec<u8>, value: &str) {
	let bytes = value.as_bytes();
	let length = bytes.len();
	if length > 31 {
		panic!("Attempted to write a string of length {} with flex type, but only 31 characters are possible.", length);
	}
	buffer.push(0b10100000 | length as u8);
	buffer.extend(bytes.iter());
}

pub fn write_string_8(buffer: &mut Vec<u8>, value: &str) {
	let bytes = value.as_bytes();
	let length = bytes.len();
	if length > 0xFF {
		panic!("Attempted to write a string of length {} with flex type, but only 0xFF characters are possible.", length);
	}
	buffer.push(0xD9);
	buffer.push(length as u8);
	buffer.extend(bytes.iter());
}

pub fn write_string_16(buffer: &mut Vec<u8>, value: &str) {
	let bytes = value.as_bytes();
	let length = bytes.len();
	if length > 0xFFFF {
		panic!("Attempted to write a string of length {} with flex type, but only 0xFFFF characters are possible.", length);
	}
	buffer.push(0xDA);
	buffer.push((length >> 8) as u8);
	buffer.push(length as u8);
	buffer.extend(bytes.iter());
}

pub fn write_string_auto(buffer: &mut Vec<u8>, value: Option<&str>) {
	if value.is_none() {
		write_null(buffer);
		return;
	}
	
	let text = value.unwrap();
	let bytes = text.as_bytes();
	let length = bytes.len();
	match length {
		0..=31 => write_string_flex(buffer, text),
		0..=0xFF => write_string_8(buffer, text),
		0..=0xFFFF => write_string_16(buffer, text),
		_ => {
			panic!("String to write is too large: {}", length)
		}
	}
}

//Booleans:

pub fn write_bool(buffer: &mut Vec<u8>, value: bool) {
	if value {
		buffer.push(0xC3);
	} else {
		buffer.push(0xC2);
	}
}

pub fn write_bool_auto(buffer: &mut Vec<u8>, value: bool) {
	write_bool(buffer, value);
}

//Map:

pub fn write_map_flex(buffer: &mut Vec<u8>, value: u32) {
	if value > 15 {
		panic!("Flex maps only support up to 15 entries. Provided {}", value);
	}
	buffer.push(0x80 + value as u8);
}

pub fn write_map_auto(buffer: &mut Vec<u8>, value: u32) {
	match value {
		//Area of 4 bits:
		0..=0xF => write_map_flex(buffer, value),
		//Area of 16 bits:
		0x10..=0xFFFF => {
			buffer.push(0xDE);
			buffer.push((value >> 8) as u8);
			buffer.push(value as u8);
		}
		//Area of 32 bits:
		0x10000..=0xFFFFFFFF => {
			buffer.push(0xDF);
			buffer.push((value >> 24) as u8);
			buffer.push((value >> 16) as u8);
			buffer.push((value >> 8) as u8);
			buffer.push(value as u8);
		}
	}
}

//Array:

pub fn write_array_flex(buffer: &mut Vec<u8>, value: u32) {
	if value > 15 {
		panic!("Flex array only support up to 15 entries. Provided {}", value);
	}
	buffer.push(0x90 + value as u8);
}

pub fn write_array_auto(buffer: &mut Vec<u8>, value: u32) {
	match value {
		//Area of 4 bits:
		0..=0xF => write_array_flex(buffer, value),
		//Area of 16 bits:
		0x10..=0xFFFF => {
			buffer.push(0xDC);
			buffer.push((value >> 8) as u8);
			buffer.push(value as u8);
		}
		//Area of 32 bits:
		0x10000..=0xFFFFFFFF => {
			buffer.push(0xDD);
			buffer.push((value >> 24) as u8);
			buffer.push((value >> 16) as u8);
			buffer.push((value >> 8) as u8);
			buffer.push(value as u8);
		}
	}
}

//Binary:

pub(crate) fn write_binary(buffer: &mut Vec<u8>, value: &[u8]) {
	let length = value.len();
	match length {
		0..=0xFF => {
			buffer.push(0xC4);
			buffer.push(length as u8);
			buffer.extend(value.iter());
		}
		0x100..=0xFFFF => {
			buffer.push(0xC5);
			buffer.push((length >> 8) as u8);
			buffer.push(length as u8);
			buffer.extend(value.iter());
		}
		0x10000..=0xFFFFFFFF => {
			buffer.push(0xC6);
			buffer.push((length >> 24) as u8);
			buffer.push((length >> 16) as u8);
			buffer.push((length >> 8) as u8);
			buffer.push(length as u8);
			buffer.extend(value.iter());
		}
		_ => {
			panic!("The supplied array is too large, the maximum amount allowed must fit a 32 bit length.")
		}
	}
}