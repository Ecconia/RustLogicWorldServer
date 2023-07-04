use crate::prelude::*;

use std::path::{Path, PathBuf};
use std::fs;

use crate::util::succ::succ_parser;

pub struct WorldFolderAccess {
	world_folder: PathBuf,
	world_file: PathBuf,
	pub extra_data_folder: PathBuf,
}

impl WorldFolderAccess {
	pub fn initialize() -> EhResult<Self> {
		//>>> Get current directory:
		let current_dir = unwrap_or_return!(std::env::current_dir(), |error| {
			exception!("Error while getting current directory: ", format!("{:?}", error))
		});
		log_debug!("Running server in directory: '", current_dir.to_string_lossy(), "'");
		//Ensure data directory exists:
		if !current_dir.exists() {
			exception!("Running from a directory that does (no longer) exist.")?;
		}
		
		//>>> Get/Create current data folder:
		let data_folder = current_dir.join(Path::new("data"));
		if !data_folder.exists() {
			log_warn!("Data directory does not exist, creating it!");
			unwrap_or_return!(std::fs::create_dir(data_folder), |error| {
				exception!("Failed to create data directory: ", format!("{:?}", error))
			});
			return exception!("As the data directory was just created and this server can't create worlds yet. You have to copy a world into the ", "data", " folder. Make sure it is called '", "World", "'!");
		}
		if !data_folder.is_dir() {
			return exception!("Expected to find a ", "data", " folder inside the current directory. 'data' exists, but it is not a directory.");
		}
		
		//>>> Get world folder:
		let world_folder = data_folder.join(Path::new("World"));
		if !world_folder.exists() {
			return exception!("Expected to find '", "World", "' folder inside of the data directory. No world found, copy one here.");
		}
		if !world_folder.is_dir() {
			return exception!("Expected to find a ", "World", " folder inside of the data directory. 'data' exists, but it is not a directory.");
		}
		
		//>>> World data file:
		let world_file = world_folder.join("data.logicworld");
		if !world_file.exists() {
			return exception!("Expected to find '", "data.logicworld", "' file inside of the world directory. But the file is not there (It contains the world data - you should be concerned).");
		}
		if !world_file.is_file() {
			return exception!("Expected to find a ", "data.logicworld", " file inside the current directory. 'data.logicworld' exists, but it is not a file.");
		}
		
		//>>> Extra data folder:
		let extra_data_folder = world_folder.join(Path::new("ExtraData"));
		if !extra_data_folder.exists() {
			log_warn!("Expected to find '", "ExtraData", "' folder inside of the world directory. Not found, will create an empty directory.");
			unwrap_or_return!(std::fs::create_dir(&extra_data_folder), |error| {
				exception!("Failed to create ExtraData directory: ", format!("{:?}", error))
			});
		}
		if !extra_data_folder.is_dir() {
			return exception!("Expected to find a ", "ExtraData", " folder inside of the world directory. 'ExtraData' exists, but it is not a directory.");
		}
		
		Ok(Self {
			world_folder,
			world_file,
			extra_data_folder,
		})
	}
	
	pub fn load_world_file(&self) -> EhResult<Vec<u8>> {
		let data_vec = Self::load_file(&self.world_file).wrap(ex!("While loading world from disk"))?;
		log_debug!("Read world with ", data_vec.len(), " bytes");
		Ok(data_vec)
	}
	
	fn load_file(path: &PathBuf) -> EhResult<Vec<u8>> {
		let data_vec = unwrap_or_return!(std::fs::read(path), |error| {
			exception!("Failed to read ", path.to_string_lossy(), ": ", format!("{:?}", error))
		});
		Ok(data_vec)
	}
	
	pub fn parse_all_succ_files(&self) -> EhResult<()>{
		let root_folder = &self.world_folder;
		
		let a = &mut vec![root_folder.to_owned()];
		let b = &mut Vec::new();
		while !a.is_empty() {
			for folder_path in a.iter_mut() {
				for dir_entry in fs::read_dir(folder_path).map_ex(ex!("While getting folders from directory"))? {
					let dir_entry = dir_entry.map_ex(ex!())?.path();
					if dir_entry.is_dir() {
						b.push(dir_entry);
						continue;
					}
					if dir_entry.is_file() && dir_entry.to_string_lossy().ends_with(".succ") {
						//Process
						log_info!("Trying to parse: ", &dir_entry.to_string_lossy());
						let bytes = Self::load_file(&dir_entry).wrap(ex!("While loading SUCC file bytes from disk"))?;
						succ_parser::debug_succ_file(&bytes).wrap(ex!("While trying to parse random SUCC file"))?;
					}
				}
			}
			a.clear();
			std::mem::swap(a, b);
		}
		
		Ok(())
	}
}
