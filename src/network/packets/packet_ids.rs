//File is adjusted for LogicWorld version 0.91.X

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum PacketIDs {
	//### Join procedure: ###
	//C2S: Multiplayer and pre-join
	DiscoveryRequest = 10,
	//S2C: Answer, contains join instructions
	DiscoveryResponse = 11,
	//C2S: Send actual join request to server
	ConnectionApproval = 16,
	//(S2C): Answering with Lidgren ConnectResponse or Disconnect
	//C2S: Very first data message, indicating ready to converse
	ConnectionEstablished = 17,
	//S2C: Send the world to client
	WorldInitialization = 18,
	//C2S: Indicates that processing the world is done, and player is ready to interact
	ClientLoadedWorld = 15,
	
	//### Building: ###
	//Chat:
	ChatMessageSent = 3, //C2S
	ChatMessageBroadcast = 20, //S2Broadcast
	//Command:
	RunCommand = 14, //C2S
	//Building:
	BuildingRequest = 2, //C2S
	BuildActionReceipt = 19, //S2C: Answer to build request
	WorldUpdate = 30, //S2Broadcast: Broadcast world changes
	//ExtraData:
	ExtraDataRequest = 5, //C2S: Request some extra data to be sent -> causes answer
	ExtraDataChange = 4, //C2S: Request some extra data to be changed -> causes broadcast
	ExtraDataUpdate = 23, //S2CB: Update one or more client with the (new) extra data
	//Simulation:
	RequestSimulationSteps = 9, //C2S: Trigger stepping a tick
	ResetToDefaultSimulationSpeed = 12, //C2S: Reset TPS to default
	ServerStrugglingWithSimulationSpeed = 28, //S2Broadcast: Send the amount of missed ticks in seconds
	CircuitStatesUpdate = 21, //S2Broadcast: Update circuit state changes
	//Special:
	RpcCall = 13, //C2S: Send RPC instruction
	RpcConfirm = 27, //S2C: Confirm execution - For callback?
	//Other:
	DebugMessage = 22, //S2C: Send message to clients console
	//Subassembly:
	SubassemblyRequest = 31, //C2S: No clue!
	SubassemblyRequestResponse = 32, //S2C: Answer to no clue!
	//### Player update: ###
	//List of Players
	PlayerList = 25, //S2Broadcast: Update the list of visible Bobbys
	//Player movement:
	PlayerPosition = 8, //C2S: Tell server about the new mental position of Bobby
	SetPlayerPositionData = 29, //S2C: Update the clients position (by force)
	PlayerPositionUpdate = 26, //C2Broadcast: Tell everyone about the new position
	//Player hotbar:
	PlayerHotbar = 7, //C2S: Tell server about the new hotbar content (or so?)
	//Player looks:
	PlayerAppearance = 6, //C2S: Update server with Bobby's new look
	PlayerAppearanceUpdate = 24, //C2Broadcast: Tell everyone about the new look
}

impl PacketIDs {
	pub const fn id(&self) -> u32 {
		*self as u32
	}
	
	pub fn from_u32(value : u32) -> Option<PacketIDs> {
		if value > u8::MAX as u32{
			return None;
		}
		Some(unsafe { std::mem::transmute(value as u8) })
	}
}
