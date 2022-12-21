use std::collections::HashMap;
use std::path::Path;
use crate::files::world_data::world_structs::{Component, ComponentAddress, PegAddress, Wire, World};
use crate::prelude::*;
use crate::util::custom_iterator::CustomIterator;

const LW_FILE_HEADER: &str = "Logic World save";
const LW_FILE_FOOTER: &str = "redstone sux lol";
const LW_FILE_HEADER_BYTES: &[u8] = LW_FILE_HEADER.as_bytes();
const LW_FILE_FOOTER_BYTES: &[u8] = LW_FILE_FOOTER.as_bytes();

pub fn load_world() -> EhResult<World> {
	let current_dir = unwrap_ok_or_return!(std::env::current_dir(), |error| {
		exception!("Error while getting current directory: ", format!("{:?}", error))
	});
	log_debug!("Running server in: '", current_dir.to_string_lossy(), "'");
	if !current_dir.exists() {
		return exception!("Running from a directory that does (no longer) exist.");
	}
	
	//Create path and validate existence:
	let data_folder = current_dir.join(Path::new("data"));
	if !data_folder.exists() {
		log_warn!("Data directory does not exist, creating it!");
		unwrap_ok_or_return!(std::fs::create_dir(data_folder), |error| {
			exception!("Failed to create data directory: ", format!("{:?}", error))
		});
		return exception!("As the data directory was just created and this server can't create worlds yet. You have to copy a world into the ", "data", " folder. Make sure it is called '", "World", "'!");
	}
	if !data_folder.is_dir() {
		return exception!("Expected to find a ", "data", " folder inside the current directory. 'data' exists, but it is not a directory.");
	}
	let world_folder = data_folder.join(Path::new("World"));
	if !world_folder.exists() {
		return exception!("Expected to find '", "World", "' folder inside of the data directory. No world found, copy one here.");
	}
	if !world_folder.is_dir() {
		return exception!("Expected to find a ", "World", " folder inside of the data directory. 'data' exists, but it is not a directory.");
	}
	let world_data_file = world_folder.join("data.logicworld");
	if !world_folder.exists() {
		return exception!("Expected to find '", "data.logicworld", "' file inside of the world directory. But the file is not there (It contains the world data - you should be concerned).");
	}
	if !world_data_file.is_file() {
		return exception!("Expected to find a ", "data.logicworld", " file inside the current directory. 'data.logicworld' exists, but it is not a file.");
	}
	
	//Read the whole file into RAM:
	let data_vec = unwrap_ok_or_return!(std::fs::read(world_data_file), |error| {
		exception!("Failed to read world data file: ", format!("{:?}", error))
	});
	log_debug!("Read world with ", data_vec.len(), " bytes");
	
	let iterator = &mut CustomIterator::create(&data_vec[..]);
	return read_from_file(iterator);
}

fn read_from_file(iterator: &mut CustomIterator) -> EhResult<World> {
	if iterator.remaining() < (LW_FILE_FOOTER_BYTES.len() + LW_FILE_HEADER_BYTES.len() + 1) {
		return exception!("World data file is too small to contain any data (the header+footer+file-version do not fit).");
	}
	
	//Check header:
	{
		let header = iterator.read_slice_unchecked(LW_FILE_HEADER_BYTES.len());
		if header != LW_FILE_HEADER_BYTES {
			return exception!("World data file does not start with the logic world header: ", LW_FILE_HEADER);
		}
	}
	//Check footer:
	{
		let position = iterator.pointer_save();
		iterator.pointer_restore(0);
		iterator.pointer_restore(iterator.remaining() - LW_FILE_FOOTER_BYTES.len());
		let header = iterator.read_slice_unchecked(LW_FILE_FOOTER_BYTES.len());
		if header != LW_FILE_FOOTER_BYTES {
			return exception!("World data file does not end with the logic world footer: ", LW_FILE_FOOTER);
		}
		iterator.pointer_restore(position);
	}
	
	//### SAVE INFO: ###########
	
	let file_version = iterator.next_unchecked(); //Bounds check still covered by length check above.
	if file_version < 5 {
		return exception!("World data format too old, need at least version 5, got version ", file_version, ".");
	}
	if file_version > 6 {
		return exception!("World data format too new, need at most version 6, got version ", file_version, ". (Tell the dev to update this server, or your save file is broken).");
	}
	// log_debug!("Got supported save file version ", file_version);
	let patch_positions = file_version == 5;
	if patch_positions {
		log_warn!("Save file version is ", 5, ", converting relative positions to fixed point on the fly.")
	}
	
	let header_data_length = (4 + 4 + 4 + 4) + 1 + (4 + 4);
	if iterator.remaining() < header_data_length {
		return exception!("File not large enough to hold the basic save information. Needs at least ", header_data_length, " but only got ", iterator.remaining());
	}
	let _game_version = read_version_unchecked(iterator);
	{
		let save_type = iterator.next_unchecked();
		match save_type {
			0 => return exception!("World data indicated, that it does not know its own type (got a ", 0, " type - should be ", 1, ")."),
			1 => {} //All good.
			2 => return exception!("World data indicated, that it is a subassembly type, needs world type data though."),
			_ => return exception!("Unknown world data type ", save_type, " should be ", "1", " for world type."),
		}
	}
	let amount_components = iterator.read_le_u32().unwrap();
	let amount_wires = iterator.read_le_u32().unwrap();
	
	let amount_mods = exception_wrap!(read_semi_unsigned_int(iterator), "While reading amount of mods in save file")?;
	let mut mods = HashMap::with_capacity(amount_mods as usize);
	for _ in 0..amount_mods {
		let mod_name = exception_wrap!(read_string(iterator), "While reading mod name")?;
		let mod_version = exception_wrap!(read_version(iterator), "While reading mod version")?;
		log_debug!("Found mod entry '", mod_name, "' with version ", format!("{}.{}.{}.{}", mod_version.0, mod_version.1, mod_version.2, mod_version.3));
		mods.insert(mod_name, mod_version);
	}
	
	let components_map_count = exception_wrap!(read_semi_unsigned_int(iterator), "While reading amount of component dictionary entries")?;
	if components_map_count > 65534 {
		return exception!("Amount of different component is too large: ", components_map_count, " / ", 65534);
	}
	let mut component_dictionary = HashMap::with_capacity(components_map_count as usize);
	for _ in 0..components_map_count {
		let index = exception_wrap!(iterator.read_le_u16(), "While reading index of the ID-component mapping")?;
		let identifier = exception_wrap!(read_string(iterator), "While reading identifier of the ID-component mapping")?;
		// log_debug!("Component entry: ", index, " <= ", identifier);
		component_dictionary.insert(index, identifier);
	}
	
	//### COMPONENTS: ###########
	
	let mut components = Vec::with_capacity(amount_components as usize);
	for _ in 0..amount_components {
		let component_address = exception_wrap!(read_component_address(iterator), "While reading component address")?;
		//TODO: Check that all parent addresses actually do exists.
		let parent_address = exception_wrap!(read_component_address(iterator), "While reading component parent address")?;
		let component_type_index = exception_wrap!(iterator.read_le_u16(), "While reading component type index")?;
		let _component_type = unwrap_some_or_return!(component_dictionary.get(&component_type_index), {
			exception!("Component type ID with not entry in component-ID map found: ", component_type_index)
		});
		let relative_position = exception_wrap!(read_position(iterator, patch_positions), "While reading component position")?;
		let relative_alignment = exception_wrap!(read_alignment(iterator), "While reading component alignment")?;
		
		let amount_inputs = exception_wrap!(read_semi_unsigned_int(iterator), "While reading component input amount")?;
		let mut inputs = Vec::with_capacity(amount_inputs as usize);
		for _ in 0..amount_inputs {
			let circuit_state_id = exception_wrap!(read_semi_unsigned_int(iterator), "While reading component input circuit state id")?;
			inputs.push(circuit_state_id);
		}
		let amount_outputs = exception_wrap!(read_semi_unsigned_int(iterator), "While reading component output amount")?;
		let mut outputs = Vec::with_capacity(amount_outputs as usize);
		for _ in 0..amount_outputs {
			let circuit_state_id = exception_wrap!(read_semi_unsigned_int(iterator), "While reading component output circuit state id")?;
			outputs.push(circuit_state_id);
		}
		let amount_custom_data_bytes = exception_wrap!(iterator.read_le_i32(), "While reading custom data byte amount")?;
		if amount_custom_data_bytes < -1 {
			return exception!("Expected -1 or higher for component custom data byte amount, got: ", amount_custom_data_bytes);
		}
		let custom_data = if amount_custom_data_bytes > 0 {
			exception_wrap!(iterator.read_bytes(amount_custom_data_bytes as usize), "While reading component custom data bytes")?
		} else {
			Vec::with_capacity(0)
		};
		components.push(Component {
			address: component_address,
			parent: parent_address,
			type_id: component_type_index,
			relative_position,
			relative_alignment,
			inputs,
			outputs,
			custom_data
		});
	}
	
	//### WIRES: ################
	
	let mut wires = Vec::with_capacity(amount_wires as usize);
	let bytes_per_wire = 9 + 9 + 4 + 4;
	for _ in 0..amount_wires {
		if iterator.remaining() < bytes_per_wire {
			return exception!("Ran out of bytes while reading wire entry, safe file seems corrupted. Remaining bytes: ", iterator.remaining(), " / ", bytes_per_wire);
		}
		let peg_address_a = exception_wrap!(read_peg_address_unchecked(iterator), "While reading a wires peg address (A)")?;
		let peg_address_b = exception_wrap!(read_peg_address_unchecked(iterator), "While reading a wires peg address (B)")?;
		let circuit_state_id = exception_wrap!(read_semi_unsigned_int(iterator), "While reading a wires circuit state id")?;
		let wire_rotation = iterator.read_le_f32().unwrap(); //Bound check is done above.
		wires.push(Wire {
			peg_a: peg_address_a,
			peg_b: peg_address_b,
			circuit_state_id,
			rotation: wire_rotation,
		})
	}
	
	//### CIRCUIT STATES: #######
	
	let amount_of_bytes = exception_wrap!(read_semi_unsigned_int(iterator), "While reading amount of circuit state bytes")?;
	let mut circuit_states = Vec::with_capacity(amount_of_bytes as usize * 8);
	for byte in exception_wrap!(iterator.read_bytes(amount_of_bytes as usize), "While reading circuit state bytes")? {
		circuit_states.push(byte & 0b00000001 != 0);
		circuit_states.push(byte & 0b0000001 != 0);
		circuit_states.push(byte & 0b000001 != 0);
		circuit_states.push(byte & 0b00001 != 0);
		circuit_states.push(byte & 0b0001 != 0);
		circuit_states.push(byte & 0b001 != 0);
		circuit_states.push(byte & 0b01 != 0);
		circuit_states.push(byte & 0b1 != 0);
	}
	
	if iterator.remaining() != LW_FILE_FOOTER_BYTES.len() {
		return exception!("Expected to have read all bytes inside of world file, with only the footer being left over. But have ", iterator.remaining(), " / ", LW_FILE_FOOTER_BYTES.len(), " left.");
	}
	
	log_debug!("Finished reading the world file.");
	
	Ok(World {
		component_id_map: component_dictionary,
		components,
		wires,
		circuit_states,
	})
}

//Byte count: 9
fn read_peg_address_unchecked(iterator: &mut CustomIterator) -> EhResult<PegAddress> {
	let is_input = exception_wrap!(read_bool_unchecked(iterator), "While reading peg address type bool")?;
	let component_address = exception_wrap!(read_component_address(iterator), "While reading peg address component address")?;
	let peg_index = exception_wrap!(read_semi_unsigned_int(iterator), "While reading peg index")?;
	Ok(PegAddress {
		is_input,
		component_address,
		peg_index,
	})
}

fn read_component_address(iterator: &mut CustomIterator) -> EhResult<ComponentAddress> {
	let component_address = exception_wrap!(iterator.read_le_u32(), "While reading component address")?;
	Ok(ComponentAddress {
		id: component_address,
	})
}

fn read_bool_unchecked(iterator: &mut CustomIterator) -> EhResult<bool> {
	let bool_byte = iterator.next_unchecked();
	match bool_byte {
		0 => Ok(false),
		1 => Ok(true),
		_ => exception!("Expected boolean, but got invalid byte: ", bool_byte, " should be 0 or 1.")
	}
}

fn read_alignment(iterator: &mut CustomIterator) -> EhResult<(f32, f32, f32, f32)> {
	if iterator.remaining() < 16 {
		return exception!("Ran out of bytes, while parsing quaternion: ", iterator.remaining(), "/", 16);
	}
	Ok((
		iterator.read_le_f32().unwrap(),
		iterator.read_le_f32().unwrap(),
		iterator.read_le_f32().unwrap(),
		iterator.read_le_f32().unwrap()
	))
}

fn read_position(iterator: &mut CustomIterator, convert_from_float: bool) -> EhResult<(i32, i32, i32)> {
	if iterator.remaining() < 12 {
		return exception!("Ran out of bytes, while parsing position: ", iterator.remaining(), "/", 12);
	}
	return if convert_from_float {
		Ok((
			convert(iterator.read_le_f32().unwrap()),
			convert(iterator.read_le_f32().unwrap()),
			convert(iterator.read_le_f32().unwrap()),
		))
	} else {
		Ok((
			iterator.read_le_i32().unwrap(),
			iterator.read_le_i32().unwrap(),
			iterator.read_le_i32().unwrap(),
		))
	};
	//Method internal helper method:
	fn convert(original_float_value: f32) -> i32 {
		let fixed_unit_value = original_float_value * 1000.0; //Meters are getting converted to millimeters.
		fixed_unit_value.round() as i32 //TODO: This is not round_to_even as the C# code is doing... it should however provide similar enough results.
	}
}

fn read_semi_unsigned_int(iterator: &mut CustomIterator) -> EhResult<u32> {
	let value = exception_wrap!(iterator.read_le_i32(), "While reading semi unsigned integer")?;
	if value < 0 {
		return exception!("Expected signed integer to be positive, but got ", value);
	}
	Ok(value as u32)
}

fn read_string(iterator: &mut CustomIterator) -> EhResult<String> {
	let amount_bytes = exception_wrap!(iterator.read_le_u32(), "While reading length of string")?;
	exception_from!(String::from_utf8(exception_wrap!(iterator.read_bytes(amount_bytes as usize), "While reading string bytes")?), "While validating string bytes as string")
}

fn read_version(iterator: &mut CustomIterator) -> EhResult<(i32, i32, i32, i32)> {
	if iterator.remaining() < 16 {
		return exception!("Ran out of bytes, while parsing version: ", iterator.remaining(), "/", 16);
	}
	Ok(read_version_unchecked(iterator))
}

fn read_version_unchecked(iterator: &mut CustomIterator) -> (i32, i32, i32, i32) {
	(iterator.read_le_i32().unwrap(), iterator.read_le_i32().unwrap(), iterator.read_le_i32().unwrap(), iterator.read_le_i32().unwrap())
}
