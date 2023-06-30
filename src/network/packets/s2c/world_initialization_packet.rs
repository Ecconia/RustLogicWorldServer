use crate::files::world_data::world_structs::World;
use crate::network::message_pack::writer as mp_writer;
use crate::network::packets::packet_ids::PacketIDs;

pub struct WorldInitializationPacket<'a> {
	world: &'a mut World,
}

impl<'a> WorldInitializationPacket<'a> {
	pub fn simple(world: &mut World) -> WorldInitializationPacket {
		WorldInitializationPacket {
			world
		}
	}
	
	pub fn write(&self, buffer: &mut Vec<u8>) {
		//Version:
		mp_writer::write_int_auto(buffer, PacketIDs::WorldInitialization.id());
		
		//Data:
		{
			mp_writer::write_array_auto(buffer, 8);
			
			//CircuitStates:
			mp_writer::write_array_auto(buffer, self.world.circuit_states.len() as u32);
			for state in self.world.circuit_states.iter() {
				mp_writer::write_bool(buffer, *state);
			}
			
			//ComponentIDsMap:
			mp_writer::write_map_auto(buffer, self.world.component_id_map.len() as u32);
			for (id, name) in self.world.component_id_map.iter() {
				mp_writer::write_int_16(buffer, *id);
				mp_writer::write_string_auto(buffer, Some(name));
			}
			
			//WorldTypeID:
			mp_writer::write_string_auto(buffer, Some("MHG.Gridlands")); //MHG.Grasslands
			
			//Components:
			mp_writer::write_array_auto(buffer, self.world.components.len() as u32); //No components
			for component in self.world.components.iter() {
				//Tuple declaration:
				mp_writer::write_array_auto(buffer, 2);
				mp_writer::write_array_auto(buffer, 1); //To wrap the component address...
				mp_writer::write_int_auto(buffer, component.address.id);
				
				mp_writer::write_array_auto(buffer, 7);
				
				//Type:
				mp_writer::write_array_auto(buffer, 1); //To wrap...
				mp_writer::write_int_16(buffer, component.type_id);
				
				//Inputs:
				mp_writer::write_array_auto(buffer, component.inputs.len() as u32);
				for state in component.inputs.iter() {
					mp_writer::write_array_auto(buffer, 1); //To wrap...
					mp_writer::write_int_auto(buffer, *state);
				}
				
				//Outputs:
				mp_writer::write_array_auto(buffer, component.outputs.len() as u32);
				for state in component.outputs.iter() {
					mp_writer::write_array_auto(buffer, 1); //To wrap...
					mp_writer::write_int_auto(buffer, *state);
				}
				
				//CustomData:
				//TODO: NULLABLE
				mp_writer::write_binary(buffer, &component.custom_data);
				
				//Parent:
				mp_writer::write_array_auto(buffer, 1); //To wrap...
				mp_writer::write_int_auto(buffer, component.parent.id);
				
				//RelativePosition:
				mp_writer::write_array_auto(buffer, 3);
				mp_writer::write_i32(buffer, component.relative_position.0);
				mp_writer::write_i32(buffer, component.relative_position.1);
				mp_writer::write_i32(buffer, component.relative_position.2);
				
				//RelativeAlignment:
				mp_writer::write_array_auto(buffer, 4);
				mp_writer::write_float_auto(buffer, component.relative_alignment.0);
				mp_writer::write_float_auto(buffer, component.relative_alignment.1);
				mp_writer::write_float_auto(buffer, component.relative_alignment.2);
				mp_writer::write_float_auto(buffer, component.relative_alignment.3);
			}
			
			//Wires:
			mp_writer::write_map_auto(buffer, self.world.wires.len() as u32);
			let mut index = 1;
			for wire in self.world.wires.iter() {
				mp_writer::write_array_auto(buffer, 1); //To wrap the wire address...
				mp_writer::write_int_auto(buffer, index);
				index += 1;
				
				mp_writer::write_array_auto(buffer, 4);
				
				//Peg 1:
				mp_writer::write_array_auto(buffer, 2);
				mp_writer::write_int_auto(buffer, (!wire.peg_a.is_input) as u32);
				mp_writer::write_array_auto(buffer, 2);
				mp_writer::write_array_auto(buffer, 1); //To wrap...
				mp_writer::write_int_auto(buffer, wire.peg_a.component_address.id);
				mp_writer::write_int_auto(buffer, wire.peg_a.peg_index);
				
				//Peg 2:
				mp_writer::write_array_auto(buffer, 2);
				mp_writer::write_int_auto(buffer, (!wire.peg_b.is_input) as u32);
				mp_writer::write_array_auto(buffer, 2);
				mp_writer::write_array_auto(buffer, 1); //To wrap...
				mp_writer::write_int_auto(buffer, wire.peg_b.component_address.id);
				mp_writer::write_int_auto(buffer, wire.peg_b.peg_index);
				
				//Circuit state:
				mp_writer::write_int_auto(buffer, wire.circuit_state_id);
				
				//Rotation:
				mp_writer::write_float_auto(buffer, wire.rotation);
			}
			
			//PlayerPosition:
			{
				mp_writer::write_array_auto(buffer, 7);
				//BaseWorldRotation:
				{
					mp_writer::write_array_auto(buffer, 4);
					//Data:
					mp_writer::write_float_auto(buffer, 0.0);
					mp_writer::write_float_auto(buffer, 0.0);
					mp_writer::write_float_auto(buffer, 0.0);
					mp_writer::write_float_auto(buffer, 1.0);
				}
				//FeetPosition:
				{
					mp_writer::write_array_auto(buffer, 3);
					//Data:
					mp_writer::write_float_auto(buffer, 0.0);
					mp_writer::write_float_auto(buffer, 1.0);
					mp_writer::write_float_auto(buffer, 0.0);
				}
				//HeadHorizontalRotation:
				mp_writer::write_float_auto(buffer, 0.0);
				//HeadVerticalRotation:
				mp_writer::write_float_auto(buffer, 0.0);
				//Scale:
				mp_writer::write_float_auto(buffer, 1.0);
				//Flying:
				mp_writer::write_bool_auto(buffer, true);
				//TeleportFromPreviousPosition:
				mp_writer::write_bool_auto(buffer, false);
			}
			
			//PlayerHotbar:
			mp_writer::write_null(buffer); //No clue if a "null" hotbar works, but I think that means default.
			
			//PlayerName:
			mp_writer::write_string_auto(buffer, Some("EpicUsername"));
		}
	}
}
