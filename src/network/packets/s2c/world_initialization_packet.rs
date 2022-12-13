use crate::network::message_pack::writer as mp_writer;
use crate::network::packets::packet_ids::PacketIDs;

pub struct WorldInitializationPacket {}

impl WorldInitializationPacket {
	pub fn simple() -> WorldInitializationPacket {
		WorldInitializationPacket {}
	}
	
	pub fn write(&self, buffer: &mut Vec<u8>) {
		//Version:
		mp_writer::write_int_auto(buffer, PacketIDs::WorldInitializationPacket.id());
		
		//Data:
		{
			mp_writer::write_array_auto(buffer, 8);
			//CircuitStates:
			mp_writer::write_array_auto(buffer, 0); //No circuit states for now...
			//ComponentIDsMap:
			mp_writer::write_map_auto(buffer, 1); //Only pegs here!
			mp_writer::write_int_auto(buffer, 0); //Peg ID shall be 0
			mp_writer::write_string_auto(buffer, Some("MHG.Peg"));
			//WorldTypeID:
			mp_writer::write_string_auto(buffer, Some("MHG.Grasslands"));
			//Components:
			mp_writer::write_array_auto(buffer, 0); //No components
			//Wires:
			mp_writer::write_map_auto(buffer, 0); //No wires
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
