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
		let current_dir = unwrap_or_else_return!(std::env::current_dir(), |error| {
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
			unwrap_or_else_return!(std::fs::create_dir(data_folder), |error| {
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
			unwrap_or_else_return!(std::fs::create_dir(&extra_data_folder), |error| {
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
	
	pub fn load_file(path: &PathBuf) -> EhResult<Vec<u8>> {
		let data_vec = unwrap_or_else_return!(std::fs::read(path), |error| {
			exception!("Failed to read ", path.to_string_lossy(), ": ", format!("{:?}", error))
		});
		Ok(data_vec)
	}
}

impl WorldFolderAccess {
	pub fn iterate_files_of_type<T, F: Fn(&mut T, String, PathBuf) -> EhResult<()>>(
		entry_folder: &PathBuf,
		file_type: &str,
		instance: &mut T,
		closure: F
	) -> EhResult<()> {
		let file_prefix = entry_folder.to_string_lossy().len() + 1;
		let mut stack = vec![fs::read_dir(entry_folder).unwrap()];
		
		let possible_next_element = stack.last_mut().unwrap().next();
		if possible_next_element.is_none() {
			return Ok(()); //Nothing to do.
		}
		let mut element = possible_next_element.unwrap();
		loop {
			let entry = element.as_ref().map_ex(ex!("While trying to read file information from folder"))?.path();
			if entry.is_dir() {
				stack.push(fs::read_dir(entry).unwrap());
			} else {
				let name = entry.to_string_lossy();
				if name.ends_with(file_type) {
					let key = name.chars().into_iter().skip(file_prefix).take(name.len() - file_type.len() - file_prefix).collect::<String>();
					closure(instance, key, entry)?;
				}
			}
			//Take the next iterator/element:
			loop {
				let next_relevant_iterator = stack.last_mut();
				if let Some(iterator) = next_relevant_iterator {
					//Got an iterator/folder:
					let possible_next_element = iterator.next();
					if let Some(el) = possible_next_element {
						//Got valid element:
						element = el;
						break; //Continue with outer loop.
					}
					//No more element in this iterator/folder, skip and get next:
					stack.pop();
				} else {
					//No more iterator/folder, stop here.
					return Ok(());
				}
			}
		}
	}
	
	pub fn parse_all_succ_files(&self) -> EhResult<()>{
		Self::iterate_files_of_type(
			&self.world_folder,
			".succ",
			&mut 0u8,
			|_, key, path| {
				//Process
				log_info!("Trying to parse: ", key);
				let bytes = Self::load_file(&path).wrap(ex!("While loading SUCC file bytes from disk"))?;
				succ_parser::debug_succ_file(&bytes).wrap(ex!("While trying to parse random SUCC file"))?;
				Ok(())
			},
		)?;
		Ok(())
	}
	
	pub fn iterate_extra_data<T, F: Fn(&mut T, String, PathBuf) -> EhResult<()>>(
		&self,
		instance: &mut T,
		closure: F
	) -> EhResult<()> {
		Self::iterate_files_of_type(
			&self.extra_data_folder,
			".succ",
			instance,
			closure,
		)?;
		Ok(())
	}
}
