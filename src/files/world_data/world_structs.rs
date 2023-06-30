use std::collections::HashMap;

pub struct World {
	pub component_id_map: HashMap<u16, String>,
	pub components: Vec<Component>,
	pub wires: Vec<Wire>,
	pub circuit_states: Vec<bool>,
}

pub struct Component {
	pub address: ComponentAddress,
	pub parent: ComponentAddress,
	pub type_id: u16,
	pub relative_position: (i32, i32, i32),
	pub relative_alignment: (f32, f32, f32, f32),
	pub inputs: Vec<u32>,
	pub outputs: Vec<u32>,
	pub custom_data: Vec<u8>,
}

pub struct Wire {
	pub peg_a: PegAddress,
	pub peg_b: PegAddress,
	pub circuit_state_id: u32,
	pub rotation: f32,
}

pub struct PegAddress {
	pub is_input: bool,
	pub component_address: ComponentAddress,
	pub peg_index: u32,
}

pub struct ComponentAddress {
	pub id: u32,
}

//Not directly world, but part of CustomData and (probably) more:

#[derive(Default)]
pub struct Color24 {
	pub r: u8,
	pub g: u8,
	pub b: u8,
}
