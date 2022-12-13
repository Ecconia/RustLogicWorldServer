//File is adjusted for LogicWorld version 0.91.X
use PacketIDs::*;

pub enum PacketIDs {
	//### Join procedure: ###
	//C2S: Multiplayer and pre-join
	DiscoveryRequestPacket,
	//S2C: Answer, contains join instructions
	DiscoveryResponsePacket,
	//C2S: Send actual join request to server
	ConnectionApprovalPacket,
	//(S2C): Answering with Lidgren ConnectResponse or Disconnect
	//C2S: Very first data message, indicating ready to converse
	ConnectionEstablishedPacket,
	//S2C: Send the world to client
	WorldInitializationPacket,
	//C2S: Indicates that processing the world is done, and player is ready to interact
	ClientLoadedWorldPacket,
	
	//### Building: ###
	//Chat:
	ChatMessageSentPacket, //C2S
	ChatMessageBroadcastPacket, //S2Broadcast
	//Command
	RunCommandPacket, //C2S
	//Building:
	BuildingRequestPacket, //C2S
	BuildActionReceiptPacket, //S2C: Answer to build request
	WorldUpdatePacket, //S2Broadcast: Broadcast world changes
	//ExtraData:
	ExtraDataRequestPacket, //C2S: Request some extra data to be sent -> causes answer
	ExtraDataChangePacket, //C2S: Request some extra data to be changed -> causes broadcast
	ExtraDataUpdatePacket, //S2CB: Update one or more client with the (new) extra data
	//Simulation:
	RequestSimulationStepsPacket, //C2S: Trigger stepping a tick
	ResetToDefaultSimulationSpeedPacket, //C2S: Reset TPS to default
	ServerStrugglingWithSimulationSpeedPacket, //S2Broadcast: Send the amount of missed ticks in seconds
	CircuitStatesUpdatePacket, //S2Broadcast: Update circuit state changes
	//Special:
	RpcCallPacket, //C2S: Send RPC instruction
	RpcConfirmPacket, //S2C: Confirm execution - For callback?
	//Other:
	DebugMessagePacket, //S2C: Send message to clients console
	//Subassembly:
	SubassemblyRequestPacket, //C2S: No clue!
	SubassemblyRequestResponsePacket, //S2C: Answer to no clue!
	//### Player update: ###
	//List of Players
	PlayerListPacket, //S2Broadcast: Update the list of visible Bobbys
	//Player movement:
	PlayerPositionPacket, //C2S: Tell server about the new mental position of Bobby
	SetPlayerPositionDataPacket, //S2C: Update the clients position (by force)
	PlayerPositionUpdatePacket, //C2Broadcast: Tell everyone about the new position
	//Player hotbar:
	PlayerHotbarPacket, //C2S: Tell server about the new hotbar content (or so?)
	//Player looks:
	PlayerAppearancePacket, //C2S: Update server with Bobby's new look
	PlayerAppearanceUpdatePacket, //C2Broadcast: Tell everyone about the new look
}

impl PacketIDs {
	pub const fn id(&self) -> u32 {
		match self {
			BuildingRequestPacket => 2,
			ChatMessageSentPacket => 3,
			ExtraDataChangePacket => 4,
			ExtraDataRequestPacket => 5,
			PlayerAppearancePacket => 6,
			PlayerHotbarPacket => 7,
			PlayerPositionPacket => 8,
			RequestSimulationStepsPacket => 9,
			DiscoveryRequestPacket => 10,
			DiscoveryResponsePacket => 11,
			ResetToDefaultSimulationSpeedPacket => 12,
			RpcCallPacket => 13,
			RunCommandPacket => 14,
			ClientLoadedWorldPacket => 15,
			ConnectionApprovalPacket => 16,
			ConnectionEstablishedPacket => 17,
			WorldInitializationPacket => 18,
			BuildActionReceiptPacket => 19,
			ChatMessageBroadcastPacket => 20,
			CircuitStatesUpdatePacket => 21,
			DebugMessagePacket => 22,
			ExtraDataUpdatePacket => 23,
			PlayerAppearanceUpdatePacket => 24,
			PlayerListPacket => 25,
			PlayerPositionUpdatePacket => 26,
			RpcConfirmPacket => 27,
			ServerStrugglingWithSimulationSpeedPacket => 28,
			SetPlayerPositionDataPacket => 29,
			WorldUpdatePacket => 30,
			SubassemblyRequestPacket => 31,
			SubassemblyRequestResponsePacket => 32,
		}
	}
}