use std::net::UdpSocket;

use crate::message_pack_reader::MessagePackReader;
use crate::message_pack_writer::MessagePackWriter;
use crate::lidgren_formatter::LidgrenFormatter;

mod buffer_reader;
mod message_pack_reader;
mod message_pack_writer;
mod lidgren_formatter;

fn main() -> std::io::Result<()> {
	{
		let socket = UdpSocket::bind("127.0.0.1:43531")?;
		
		let mut buf = [0; 0xFFFF];
		
		loop
		{
			println!("====================================");
			let (buffer_amount, remote_address) = socket.recv_from(&mut buf)?;
			println!("Received UDP packet from \x1b[38;2;255;0;150m{}\x1b[m port \x1b[38;2;255;0;150m{}\x1b[m size \x1b[38;2;255;0;150m{}\x1b[m", remote_address.ip(), remote_address.port(), buffer_amount);
			
			if buffer_amount < 5
			{
				println!("\033[38;2;255;0;0m -> PACKET TOO SHORT\033[m");
				continue;
			}
			
			let mut buffer_iterator = buf[0..buffer_amount].iter().peekable();
			
			let message_type_id = *buffer_iterator.next().unwrap();
			let fragment = (**buffer_iterator.peek().unwrap() & 1) == 1;
			let sequence_number = (*buffer_iterator.next().unwrap() as u16 >> 1) | ((*buffer_iterator.next().unwrap() as u16) << 7);
			let bits_in_message = *buffer_iterator.next().unwrap() as u16 | ((*buffer_iterator.next().unwrap() as u16) << 8);
			let bytes = (bits_in_message + 7) / 8;
			println!("Type: \x1b[38;2;255;0;150m{}\x1b[m Fragment: \x1b[38;2;255;0;150m{}\x1b[m Sequence#: \x1b[38;2;255;0;150m{}\x1b[m Bits: \x1b[38;2;255;0;150m{}\x1b[m Bytes: \x1b[38;2;255;0;150m{}\x1b[m", message_type_id, fragment, sequence_number, bits_in_message, bytes);
			//Read 5 bytes.
			
			let remaining = buffer_amount - 5;
			if remaining < bytes as usize
			{
				println!("Not enough bytes in packet. Expected {}, but got {}", bytes, buffer_amount - 5);
				continue;
			}
			
			if message_type_id == 136
			{
				println!("=> DISCOVERY!");
				//Discovery packet!
				
				{
					let packet_id = MessagePackReader::read_int_auto(&mut buffer_iterator);
					if packet_id != 12
					{
						println!("Discovery packet not from a 0.91 client, but {}, bye!", packet_id);
						continue;
					}
				}
				
				{
					let map_size = MessagePackReader::read_map_auto(&mut buffer_iterator);
					if map_size != 2
					{
						println!("While parsing discovery packet, expected map of size 2, but got {}", map_size);
						continue;
					}
				}
				
				{
					let key = MessagePackReader::read_string_auto(&mut buffer_iterator);
					if key.is_none()
					{
						println!("While parsing discovery packet, expected first map key to be present, but got null.");
						continue;
					}
					let key_sane = key.unwrap();
					if String::from("ForConnection").ne(&key_sane)
					{
						println!("While parsing discovery packet, expected first map key to be 'ForConnection', but got '{}'.", key_sane);
						continue;
					}
				}
				
				let bool = MessagePackReader::read_bool_auto(&mut buffer_iterator);
				println!("Wants to connect: \x1b[38;2;255;0;150m{}\x1b[m", bool);
				
				{
					let key = MessagePackReader::read_string_auto(&mut buffer_iterator);
					if key.is_none()
					{
						println!("While parsing discovery packet, expected first map key to be present, but got null.");
						continue;
					}
					let key_sane = key.unwrap();
					if String::from("RequestGUID").ne(&key_sane)
					{
						println!("While parsing discovery packet, expected first map key to be 'RequestGUID', but got '{}'.", key_sane);
						continue;
					}
				}
				
				let uuid;
				{
					let uuid_optional = MessagePackReader::read_string_auto(&mut buffer_iterator);
					if uuid_optional.is_none()
					{
						println!("While parsing discovery packet, expected second value to be a string, but got null.");
						continue;
					}
					uuid = uuid_optional.unwrap();
					println!("Request UUID is: \x1b[38;2;255;0;150m{}\x1b[m", uuid);
				}
				
				//Answer:
				
				let mut result_buffer = Vec::new();
				
				result_buffer.push(137);
				result_buffer.push(0);
				result_buffer.push(0);
				result_buffer.push(0);
				result_buffer.push(0);
				MessagePackWriter::write_int_auto(&mut result_buffer, 13);
				MessagePackWriter::write_map_auto(&mut result_buffer, 9);
				MessagePackWriter::write_string_auto(&mut result_buffer, Some(String::from("ServerVersion")));
				MessagePackWriter::write_string_auto(&mut result_buffer, Some(String::from("0.91.0.485")));
				MessagePackWriter::write_string_auto(&mut result_buffer, Some(String::from("RequestGuid")));
				MessagePackWriter::write_string_auto(&mut result_buffer, Some(uuid));
				MessagePackWriter::write_string_auto(&mut result_buffer, Some(String::from("HasDiscoveryInfo")));
				MessagePackWriter::write_bool(&mut result_buffer, true);
				MessagePackWriter::write_string_auto(&mut result_buffer, Some(String::from("Challenge")));
				MessagePackWriter::write_string_auto(&mut result_buffer, None);
				MessagePackWriter::write_string_auto(&mut result_buffer, Some(String::from("MOTD")));
				MessagePackWriter::write_string_auto(&mut result_buffer, Some(String::from("Rust server does NOT welcome you :)")));
				MessagePackWriter::write_string_auto(&mut result_buffer, Some(String::from("PlayersConnectedCount")));
				MessagePackWriter::write_int_auto(&mut result_buffer, 0);
				MessagePackWriter::write_string_auto(&mut result_buffer, Some(String::from("MaxPlayerCapacity")));
				MessagePackWriter::write_int_auto(&mut result_buffer, 666);
				MessagePackWriter::write_string_auto(&mut result_buffer, Some(String::from("ConnectionRequiresPassword")));
				MessagePackWriter::write_bool(&mut result_buffer, false);
				MessagePackWriter::write_string_auto(&mut result_buffer, Some(String::from("ServerRunningInVerifiedMode")));
				MessagePackWriter::write_bool(&mut result_buffer, false);
				
				let size = (result_buffer.len() - 5) * 8;
				result_buffer[3] = size as u8;
				result_buffer[4] = (size >> 8) as u8;
				
				let len = socket.send_to(&result_buffer, remote_address).unwrap();
				println!("{} bytes sent", len);
			} else if message_type_id == 131 {
				//Connect!
				println!("=> CONNECT!");
				
				let splice = &buf[5..buffer_amount];
				println!("MessageBytes: {:?}", splice);
				println!("MessageBytes: {:x?}", splice);
				{
					let app_id = LidgrenFormatter::read_string(&mut buffer_iterator);
					println!("App ID: '\x1b[38;2;255;0;150m{}\x1b[m'", app_id);
					let remote_id = LidgrenFormatter::read_int_64(&mut buffer_iterator);
					println!("Remote ID: \x1b[38;2;255;0;150m{}\x1b[m", remote_id);
					let remote_time = LidgrenFormatter::read_float(&mut buffer_iterator);
					println!("Remote time: \x1b[38;2;255;0;150m{}\x1b[m", remote_time);
				}
			} else {
				println!("Unknown message type!");
				continue;
			}
		}
	}
	// Ok(())
}
