use crate::prelude::*;

use crate::network::message_pack::reader;
use crate::util::custom_iterator::CustomIterator;

macro_rules! c_struct {
	() => { $crate::util::ansi_constants::ansi_rgb!(100, 100, 100) };
}

macro_rules! c_text {
	() => { $crate::util::ansi_constants::ansi_rgb!(180, 180, 180) };
}

macro_rules! c_val {
	() => { $crate::util::ansi_constants::ansi_rgb!(255, 100, 200) };
}

macro_rules! c_res {
	() => { $crate::util::ansi_constants::ansi_reset!() };
}

macro_rules! print_data {
	($pointless:ident, $data_type:expr, $field_type:expr, $value:expr) => {
		println!(concat!("{}", c_text!(), $data_type, ": ", c_val!(), "{}", c_text!(), " [", $field_type, "]", c_res!()), $pointless, $value);
	};
	(str $pointless:ident, $data_type:expr, $field_type:expr, $value:expr) => {
		println!(concat!("{}", c_text!(), $data_type, ": '", c_val!(), "{}", c_text!(), "' [", $field_type, "]", c_res!()), $pointless, $value);
	};
	(ext $pointless:ident, $data_type:expr, $field_type:expr, $type:expr, $bytes:expr) => {
		println!(concat!("{}", c_text!(), "Ext type ", c_val!(), "{}", c_text!(), ": ", c_val!(), "{}", c_text!(), " [", $field_type, "]", c_res!()), $pointless, $type, format!("{:?}", $bytes));
	};
}

pub fn pretty_print(iterator: &mut CustomIterator) {
	let iterator_position = iterator.pointer_save();
	
	log_debug!("");
	log_debug!("Printing MessagePack packet:");
	do_printing(iterator);
	log_debug!("");
	
	iterator.pointer_restore(iterator_position);
}

fn do_printing(iterator: &mut CustomIterator) {
	match reader::read_int_auto(iterator) {
		Err(err) => {
			log_error!("Failed to read LogicWorld packet id:");
			err.print();
			return;
		}
		Ok(id) => {
			log_debug!("Packet id: ", id);
		}
	}
	unwrap_or_print_return!(parse_entry(iterator,
	            concat!(c_struct!(), " └"),
	            concat!(c_struct!(), "  "),
	));
	if iterator.has_more() {
		log_error!("Still bytes left to read ", iterator.remaining());
	}
}

fn parse_entry(
	iterator: &mut CustomIterator,
	prefix_first: &str,
	prefix_other: &str,
) -> EhResult<()> {
	let type_fml = exception_wrap!(iterator.peek(), "While peeking next expected type byte")?;
	match type_fml {
		0x00..=0x7f => {
			//Positive fix integer:
			//Reuse the current type byte!
			let number = iterator.next_unchecked() & 0x7F;
			print_data!(prefix_first, "Int", "PosFixInt", number);
		}
		0x80..=0x8F => {
			//Fix map
			//Reuse the current byte!
			let amount = reader::read_map_flex(iterator).unwrap();
			//Should never error, as we already read that byte
			print_data!(prefix_first, "Map", "FixMap", amount);
			exception_wrap!(read_map_objects(iterator, amount as usize, prefix_other), "While iterating over FixMap entries")?;
		}
		0x90..=0x9F => {
			//Fix array
			//Reuse the current byte!
			let amount = reader::read_array_flex(iterator).unwrap();
			//Should never error, as we already read that byte
			print_data!(prefix_first, "Array", "FixArray", amount);
			exception_wrap!(read_array_objects(iterator, amount as usize, prefix_other), "While iterating over FixArray entries")?;
		}
		0xA0..=0xBF => {
			//Fix string
			//Reuse the current byte!
			let text = exception_wrap!(reader::read_string_flex(iterator), "")?;
			print_data!(str prefix_first, "String", "FixString", text);
		}
		0xC0 => {
			//Nil
			iterator.skip(); //Reuse the current byte!
			println!(concat!("{}", c_val!(), "Null", c_res!()), prefix_first);
		}
		0xC1 => {
			//NEVER USED!
			iterator.skip();
			//No clue what to do with this byte.
			log_error!("Encountered ", "NEVER USED", " entry! No clue how to parse this.");
		}
		0xC2 => {
			//False
			iterator.skip();
			//Reuse the current byte!
			println!(concat!("{}", c_val!(), "False", c_res!()), prefix_first);
		}
		0xC3 => {
			//True
			iterator.skip();
			//Reuse the current byte!
			println!(concat!("{}", c_val!(), "True", c_res!()), prefix_first);
		}
		0xC4 => {
			//Binary 8
			iterator.skip(); //The reader expects this byte to be read already...
			let bytes = exception_wrap!(reader::read_binary_8(iterator), "While reading 8BitByteArray")?;
			print_data!(prefix_first, "Bytes", "8BitByteArray", format!("{:?}", bytes));
		}
		0xC5 => {
			//Binary 16
			iterator.skip(); //The reader expects this byte to be read already...
			let bytes = exception_wrap!(reader::read_binary_16(iterator), "While reading 16BitByteArray")?;
			print_data!(prefix_first, "Bytes", "16BitByteArray", format!("{:?}", bytes));
		}
		0xC6 => {
			//Binary 32
			iterator.skip(); //The reader expects this byte to be read already...
			let bytes = exception_wrap!(reader::read_binary_32(iterator), "While reading 32BitByteArray")?;
			print_data!(prefix_first, "Bytes", "32BitByteArray", format!("{:?}", bytes));
		}
		0xC7 => {
			//Ext 8
			iterator.skip(); //Already fully used.
			let length = exception_wrap!(reader::read_int_8(iterator), "While reading 8Ext length")?;
			let sub_type = exception_wrap!(iterator.next(), "While reading 8Ext type")?;
			let data = exception_wrap!(iterator.read_bytes(length as usize), "While reading 8Ext bytes")?;
			print_data!(ext prefix_first, "Ext", "8Ext", sub_type, data);
		}
		0xC8 => {
			//Ext 16
			iterator.skip(); //Already fully used.
			let length = exception_wrap!(reader::read_int_16(iterator), "While reading 16Ext length")?;
			let sub_type = exception_wrap!(iterator.next(), "While reading 16Ext type")?;
			let data = exception_wrap!(iterator.read_bytes(length as usize), "While reading 16Ext bytes")?;
			print_data!(ext prefix_first, "Ext", "16Ext", sub_type, data);
		}
		0xC9 => {
			//Ext 32
			iterator.skip(); //Already fully used.
			let length = exception_wrap!(reader::read_int_32(iterator), "While reading 32Ext length")?;
			let sub_type = exception_wrap!(iterator.next(), "While reading 32Ext type")?;
			let data = exception_wrap!(iterator.read_bytes(length as usize), "While reading 32Ext bytes")?;
			print_data!(ext prefix_first, "Ext", "32Ext", sub_type, data);
		}
		0xCA => {
			//Float 32
			iterator.skip(); //The reader expects this byte to be read already...
			let number = exception_wrap!(reader::read_float_32(iterator), "While reading 32BitFloat")?;
			print_data!(prefix_first, "Float", "32BitFloat", number);
		}
		0xCB => {
			//Float 64
			iterator.skip(); //The reader expects this byte to be read already...
			let number = exception_wrap!(reader::read_float_64(iterator), "While reading 64BitFloat")?;
			print_data!(prefix_first, "Float", "64BitFloat", number);
		}
		0xCC => {
			//Unsigned Integer 8
			iterator.skip(); //The reader expects this byte to be read already...
			let number = exception_wrap!(reader::read_int_8(iterator), "While reading 8UInt")?;
			print_data!(prefix_first, "Int", "8UInt", number);
		}
		0xCD => {
			//Unsigned Integer 16
			iterator.skip(); //The reader expects this byte to be read already...
			let number = exception_wrap!(reader::read_int_16(iterator), "While reading 16UInt")?;
			print_data!(prefix_first, "Int", "16UInt", number);
		}
		0xCE => {
			//Unsigned Integer 32
			iterator.skip(); //The reader expects this byte to be read already...
			let number = exception_wrap!(reader::read_int_32(iterator), "While reading 32UInt")?;
			print_data!(prefix_first, "Int", "32UInt", number);
		}
		0xCF => {
			//Unsigned Integer 64
			iterator.skip(); //The reader expects this byte to be read already...
			let number = exception_wrap!(reader::read_int_64(iterator), "While reading 64UInt")?;
			print_data!(prefix_first, "Int", "64UInt", number);
		}
		0xD0 => {
			//Signed Integer 8
			iterator.skip(); //The reader expects this byte to be read already...
			let number = exception_wrap!(reader::read_sint_8(iterator), "While reading 8SInt")?;
			print_data!(prefix_first, "Int", "8SInt", number);
		}
		0xD1 => {
			//Signed Integer 16
			iterator.skip(); //The reader expects this byte to be read already...
			let number = exception_wrap!(reader::read_sint_16(iterator), "While reading 16SInt")?;
			print_data!(prefix_first, "Int", "16SInt", number);
		}
		0xD2 => {
			//Signed Integer 32
			iterator.skip(); //The reader expects this byte to be read already...
			let number = exception_wrap!(reader::read_sint_32(iterator), "While reading 32SInt")?;
			print_data!(prefix_first, "Int", "32SInt", number);
		}
		0xD3 => {
			//Signed Integer 64
			iterator.skip(); //The reader expects this byte to be read already...
			let number = exception_wrap!(reader::read_sint_64(iterator), "While reading 64SInt")?;
			print_data!(prefix_first, "Int", "64SInt", number);
		}
		0xD4 => {
			//Fix Ext 1
			iterator.skip(); //Already fully used.
			let sub_type = exception_wrap!(iterator.next(), "While reading FixExt1 type")?;
			let data = exception_wrap!(iterator.read_bytes(1), "While reading FixExt2 bytes")?;
			print_data!(ext prefix_first, "Ext", "FixExt1", sub_type, data);
		}
		0xD5 => {
			//Fix Ext 2
			iterator.skip(); //Already fully used.
			let sub_type = exception_wrap!(iterator.next(), "While reading FixExt2 type")?;
			let data = exception_wrap!(iterator.read_bytes(2), "While reading FixExt2 bytes")?;
			print_data!(ext prefix_first, "Ext", "FixExt2", sub_type, data);
		}
		0xD6 => {
			//Fix Ext 4
			iterator.skip(); //Already fully used.
			let sub_type = exception_wrap!(iterator.next(), "While reading FixExt4 type")?;
			let data = exception_wrap!(iterator.read_bytes(4), "While reading FixExt4 bytes")?;
			print_data!(ext prefix_first, "Ext", "FixExt4", sub_type, data);
		}
		0xD7 => {
			//Fix Ext 8
			iterator.skip(); //Already fully used.
			let sub_type = exception_wrap!(iterator.next(), "While reading FixExt8 type")?;
			let data = exception_wrap!(iterator.read_bytes(8), "While reading FixExt8 bytes")?;
			print_data!(ext prefix_first, "Ext", "FixExt8", sub_type, data);
		}
		0xD8 => {
			//Fix Ext 16
			iterator.skip(); //Already fully used.
			let sub_type = exception_wrap!(iterator.next(), "While reading FixExt16 type")?;
			let data = exception_wrap!(iterator.read_bytes(16), "While reading FixExt16 bytes")?;
			print_data!(ext prefix_first, "Ext", "FixExt16", sub_type, data);
		}
		0xD9 => {
			//String 8
			iterator.skip(); //The reader expects this byte to be read already...
			let text = exception_wrap!(reader::read_string_8(iterator), "While reading 8BitLengthString")?;
			print_data!(str prefix_first, "String", "8BitLengthString", text);
		}
		0xDA => {
			//String 16
			iterator.skip(); //The reader expects this byte to be read already...
			let text = exception_wrap!(reader::read_string_16(iterator), "While reading 16BitLengthString")?;
			print_data!(str prefix_first, "String", "16BitLengthString", text);
		}
		0xDB => {
			//String 32
			iterator.skip(); //The reader expects this byte to be read already...
			let text = exception_wrap!(reader::read_string_32(iterator), "While reading 32BitLengthString")?;
			print_data!(str prefix_first, "String", "32BitLengthString", text);
		}
		0xDC => {
			//Array 16
			iterator.skip(); //The reader expects this byte to be read already...
			let amount = exception_wrap!(reader::read_array_16(iterator), "While reading 16BitLengthArray amount")?;
			print_data!(prefix_first, "Array", "16BitLengthArray", amount);
			exception_wrap!(read_array_objects(iterator, amount as usize, prefix_other), "While iterating over 16BitLengthArray entries")?;
		}
		0xDD => {
			//Array 32
			iterator.skip(); //The reader expects this byte to be read already...
			let amount = exception_wrap!(reader::read_array_32(iterator), "While reading 32BitLengthArray amount")?;
			print_data!(prefix_first, "Array", "32BitLengthArray", amount);
			exception_wrap!(read_array_objects(iterator, amount as usize, prefix_other), "While iterating over 32BitLengthArray entries")?;
		}
		0xDE => {
			//Map 16
			iterator.skip(); //The reader expects this byte to be read already...
			let amount = exception_wrap!(reader::read_map_16(iterator), "While reading 16BitLengthMap amount")?;
			print_data!(prefix_first, "Map", "16BitLengthMap", amount);
			exception_wrap!(read_map_objects(iterator, amount as usize, prefix_other), "While iterating over 16BitLengthMap entries")?;
		}
		0xDF => {
			//Map 32
			iterator.skip(); //The reader expects this byte to be read already...
			let amount = exception_wrap!(reader::read_map_32(iterator), "While reading 32BitLengthMap amount")?;
			print_data!(prefix_first, "Map", "32BitLengthMap", amount);
			exception_wrap!(read_map_objects(iterator, amount as usize, prefix_other), "While iterating over 32BitLengthMap entries")?;
		}
		0xE0..=0xFF => {
			//Negative fix integer
			//Reuse the current byte!
			let number = (iterator.next_unchecked() & 0x1F | 0b11100000) as i8;
			print_data!(prefix_first, "Int", "NegFixInt", number);
		}
	}
	Ok(())
}

fn read_map_objects(iterator: &mut CustomIterator,
                    amount: usize,
                    previous_prefix: &str,
) -> EhResult<()> {
	if amount == 0 {
		return Ok(());
	}
	//Non-last entries:
	let prefix_first_key = &(previous_prefix.to_owned() + " ├")[..];
	let prefix_other_key = &(previous_prefix.to_owned() + " │ │")[..];
	let prefix_first_val = &(previous_prefix.to_owned() + " │ └")[..];
	let prefix_other_val = &(previous_prefix.to_owned() + " │  ")[..];
	for _ in 0..(amount - 1) {
		exception_wrap!(parse_entry(iterator, prefix_first_key, prefix_other_key), "While iterating over map entries (key)")?;
		exception_wrap!(parse_entry(iterator, prefix_first_val, prefix_other_val), "While iterating over map entries (value)")?;
	}
	//Last entry:
	let prefix_first_key = &(previous_prefix.to_owned() + " └")[..];
	let prefix_other_key = &(previous_prefix.to_owned() + "   │")[..];
	let prefix_first_val = &(previous_prefix.to_owned() + "   └")[..];
	let prefix_other_val = &(previous_prefix.to_owned() + "    ")[..];
	exception_wrap!(parse_entry(iterator, prefix_first_key, prefix_other_key), "While iterating over map entries (last key)")?;
	exception_wrap!(parse_entry(iterator, prefix_first_val, prefix_other_val), "While iterating over map entries (last value)")?;
	Ok(())
}

fn read_array_objects(iterator: &mut CustomIterator,
                      amount: usize,
                      previous_prefix: &str,
) -> EhResult<()> {
	if amount == 0 {
		return Ok(());
	}
	//Non-last entries:
	let prefix_first = &(previous_prefix.to_owned() + " ├")[..];
	let prefix_other = &(previous_prefix.to_owned() + " │")[..];
	for _ in 0..(amount - 1) {
		exception_wrap!(parse_entry(iterator, prefix_first, prefix_other), "While iterating over array entries")?;
	}
	//Last entry:
	let prefix_first = &(previous_prefix.to_owned() + " └")[..];
	let prefix_other = &(previous_prefix.to_owned() + "  ")[..];
	exception_wrap!(parse_entry(iterator, prefix_first, prefix_other), "While iterating over array entries (last)")?;
	Ok(())
}
