use std::net::{SocketAddr, UdpSocket};
use std::iter::Peekable;
use std::slice::Iter;

pub mod network;
pub mod lidgren;

use network::message_pack::reader as mp_reader;
use network::message_pack::writer as mp_writer;
use lidgren::util::formatter as lg_formatter;

fn main() {
	let socket = UdpSocket::bind("127.0.0.1:43531").expect("Could not bind server socket.");
	
	let mut buf = [0; 0xFFFF];
	
	loop
	{
		println!("====================================");
		let (buffer_amount, remote_address) = socket.recv_from(&mut buf).expect("Could not read incoming datagram packet.");
		println!("Received UDP packet from \x1b[38;2;255;0;150m{}\x1b[m port \x1b[38;2;255;0;150m{}\x1b[m size \x1b[38;2;255;0;150m{}\x1b[m", remote_address.ip(), remote_address.port(), buffer_amount);
		
		handle_packet(&socket, &remote_address, buffer_amount, &buf);
	}
}

fn handle_packet(socket: &UdpSocket, remote_address: &SocketAddr, buffer_amount: usize, buf: &[u8])
{
	if buffer_amount < 5
	{
		println!("\033[38;2;255;0;0m -> PACKET TOO SHORT\033[m");
		return;
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
		return;
	}
	
	if message_type_id == 136
	{
		println!("=> Discovery!");
		//Discovery packet!
		handle_discovery(&socket, &remote_address, &mut buffer_iterator);
	} else if message_type_id == 131 {
		//Connect!
		println!("=> Connect!");
		handle_connect(&socket, &remote_address, &mut buffer_iterator);
	} else if message_type_id == 133 {
		println!("=> Connection established!");
		//TODO: Read LG-Float (time)
		println!("-Cannot handle yet-");
	} else if message_type_id == 129 {
		println!("=> Ping!");
		println!("-Cannot handle yet-");
	} else if message_type_id == 67 {
		println!("=> UserReliableOrdered1!");
		//Handle actual data...
		println!("-Cannot handle yet-");
	} else {
		println!("Unknown message type!");
		return;
	}
}

fn handle_discovery(socket: &UdpSocket, remote_address: &SocketAddr, buffer_iterator: &mut Peekable<Iter<u8>>)
{
	let packet_id = mp_reader::read_int_auto(buffer_iterator);
	if packet_id != 12
	{
		println!("Discovery packet not from a 0.91 client, but {}, bye!", packet_id);
		return;
	}
	
	let map_size = mp_reader::read_map_auto(buffer_iterator);
	if map_size != 2
	{
		println!("While parsing discovery packet, expected map of size 2, but got {}", map_size);
		return;
	}
	
	let key = mp_reader::read_string_auto(buffer_iterator);
	if key.is_none()
	{
		println!("While parsing discovery packet, expected first map key to be present, but got null.");
		return;
	}
	let key_sane = key.unwrap();
	if String::from("ForConnection").ne(&key_sane)
	{
		println!("While parsing discovery packet, expected first map key to be 'ForConnection', but got '{}'.", key_sane);
		return;
	}
	
	let bool = mp_reader::read_bool_auto(buffer_iterator);
	println!("Wants to connect: \x1b[38;2;255;0;150m{}\x1b[m", bool);
	
	let key = mp_reader::read_string_auto(buffer_iterator);
	if key.is_none()
	{
		println!("While parsing discovery packet, expected first map key to be present, but got null.");
		return;
	}
	let key_sane = key.unwrap();
	if String::from("RequestGUID").ne(&key_sane)
	{
		println!("While parsing discovery packet, expected first map key to be 'RequestGUID', but got '{}'.", key_sane);
		return;
	}
	
	let uuid_optional = mp_reader::read_string_auto(buffer_iterator);
	if uuid_optional.is_none()
	{
		println!("While parsing discovery packet, expected second value to be a string, but got null.");
		return;
	}
	let uuid = uuid_optional.unwrap();
	println!("Request UUID is: \x1b[38;2;255;0;150m{}\x1b[m", uuid);
	
	//Answer:
	
	let mut result_buffer = Vec::new();
	
	result_buffer.push(137);
	result_buffer.push(0);
	result_buffer.push(0);
	result_buffer.push(0);
	result_buffer.push(0);
	mp_writer::write_int_auto(&mut result_buffer, 13);
	mp_writer::write_map_auto(&mut result_buffer, 9);
	mp_writer::write_string_auto(&mut result_buffer, Some(String::from("ServerVersion")));
	mp_writer::write_string_auto(&mut result_buffer, Some(String::from("0.91.0.485")));
	mp_writer::write_string_auto(&mut result_buffer, Some(String::from("RequestGuid")));
	mp_writer::write_string_auto(&mut result_buffer, Some(uuid));
	mp_writer::write_string_auto(&mut result_buffer, Some(String::from("HasDiscoveryInfo")));
	mp_writer::write_bool(&mut result_buffer, true);
	mp_writer::write_string_auto(&mut result_buffer, Some(String::from("Challenge")));
	mp_writer::write_string_auto(&mut result_buffer, None);
	mp_writer::write_string_auto(&mut result_buffer, Some(String::from("MOTD")));
	mp_writer::write_string_auto(&mut result_buffer, Some(String::from("Rust server does NOT welcome you :)")));
	mp_writer::write_string_auto(&mut result_buffer, Some(String::from("PlayersConnectedCount")));
	mp_writer::write_int_auto(&mut result_buffer, 0);
	mp_writer::write_string_auto(&mut result_buffer, Some(String::from("MaxPlayerCapacity")));
	mp_writer::write_int_auto(&mut result_buffer, 666);
	mp_writer::write_string_auto(&mut result_buffer, Some(String::from("ConnectionRequiresPassword")));
	mp_writer::write_bool(&mut result_buffer, false);
	mp_writer::write_string_auto(&mut result_buffer, Some(String::from("ServerRunningInVerifiedMode")));
	mp_writer::write_bool(&mut result_buffer, false);
	
	let size = (result_buffer.len() - 5) * 8;
	result_buffer[3] = size as u8;
	result_buffer[4] = (size >> 8) as u8;
	
	let len = socket.send_to(&result_buffer, remote_address).unwrap();
	println!("{} bytes sent", len);
}

fn handle_connect(socket: &UdpSocket, remote_address: &SocketAddr, buffer_iterator: &mut Peekable<Iter<u8>>)
{
	let app_id = lg_formatter::read_string(buffer_iterator);
	println!("App ID: '\x1b[38;2;255;0;150m{}\x1b[m'", app_id);
	let remote_id = lg_formatter::read_int_64(buffer_iterator);
	println!("Remote ID: \x1b[38;2;255;0;150m{}\x1b[m", remote_id);
	let remote_time = lg_formatter::read_float(buffer_iterator);
	println!("Remote time: \x1b[38;2;255;0;150m{}\x1b[m", remote_time);
	
	let packet_id = mp_reader::read_int_auto(buffer_iterator); //15
	println!("Packet ID is btw: {}", packet_id);
	
	let entry_count = mp_reader::read_array_auto(buffer_iterator);
	if entry_count != 6
	{
		println!("Client connect packet has different entry count than 6, got: {}", entry_count);
		return;
	}
	
	let mod_count = mp_reader::read_array_auto(buffer_iterator);
	println!("Mod count: {}", mod_count);
	for _ in 0..mod_count
	{
		let mod_id = mp_reader::read_string_auto(buffer_iterator);
		if mod_id.is_none()
		{
			println!("Received null mod name, illegal!");
			return;
		}
		println!(" - {}", mod_id.unwrap());
	}
	
	let user_option_count = mp_reader::read_array_auto(buffer_iterator);
	if user_option_count != 1
	{
		println!("More than one user argument, got: {}", user_option_count);
		return;
	}
	let username = mp_reader::read_string_auto(buffer_iterator);
	if username.is_none()
	{
		println!("Received null username, illegal!");
		return;
	}
	println!("Username: {}", username.unwrap());
	println!("Version: {}", mp_reader::read_string_auto(buffer_iterator).unwrap());
	println!("PWHash: {:x?}", mp_reader::read_binary_auto(buffer_iterator).unwrap());
	println!("HailPayload: {:?}", mp_reader::read_string_auto(buffer_iterator));
	println!("HailSignature: {:?}", mp_reader::read_string_auto(buffer_iterator));
	
	//Send answer:
	
	let mut result_buffer = Vec::new();
	
	result_buffer.push(132);
	result_buffer.push(0);
	result_buffer.push(0);
	result_buffer.push(0);
	result_buffer.push(0);
	
	lg_formatter::write_string(&mut result_buffer, "Logic World");
	lg_formatter::write_int_64(&mut result_buffer, remote_id);
	lg_formatter::write_float(&mut result_buffer, 0.5);
	
	let size = (result_buffer.len() - 5) * 8;
	result_buffer[3] = size as u8;
	result_buffer[4] = (size >> 8) as u8;
	
	let len = socket.send_to(&result_buffer, remote_address).unwrap();
	println!("{} bytes sent", len);
}
