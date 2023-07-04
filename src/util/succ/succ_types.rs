use crate::prelude::*;

use std::collections::HashMap;
use crate::files::world_data::world_structs::Color24;

pub enum SuccType {
	Any(), //Could be literally any of the below types, but always a length of zero
	Value(String), //Contains a single text value
	Map(HashMap<String, SuccType>), //Contains a dictionary
	List(Vec<SuccType>), //Contains a list
}

impl SuccType {
	pub fn name(&self) -> &str {
		match self {
			SuccType::Any{..} => "Any",
			SuccType::Value{..} => "Value",
			SuccType::Map{..} => "Map",
			SuccType::List{..} => "List",
			// _ => "UNKNOWN",
		}
	}
	
	pub fn is_any(&self) -> bool {
		match self {
			SuccType::Any{..} => true,
			_ => false,
		}
	}
	
	pub fn is_value(&self) -> bool {
		match self {
			SuccType::Value{..} => true,
			_ => false,
		}
	}
	
	pub fn get_value(&self) -> Option<&str> {
		if let SuccType::Value(value) = self {
			return Some(value);
		}
		return None;
	}
	
	pub fn is_map(&self) -> bool {
		match self {
			SuccType::Any{..} => true,
			SuccType::Map{..} => true,
			_ => false,
		}
	}
	
	pub fn get_map(&self) -> Option<&HashMap<String, SuccType>> {
		if let SuccType::Map(value) = self {
			return Some(value);
		}
		return None;
	}
	
	pub fn is_list(&self) -> bool {
		match self {
			SuccType::Any{..} => true,
			SuccType::List{..} => true,
			_ => false,
		}
	}
	
	pub fn get_list(&self) -> Option<&Vec<SuccType>> {
		if let SuccType::List(value) = self {
			return Some(value);
		}
		return None;
	}
}

impl SuccType {
	pub fn expect_map(&self) -> EhResult<&HashMap<String, SuccType>> {
		if !self.is_map() {
			exception!("Expected ", "MAP", " SUCC data type, got ", self.name())?;
		}
		Ok(self.get_map().unwrap())
	}
	
	pub fn expect_list(&self) -> EhResult<&Vec<SuccType>> {
		if !self.is_list() {
			exception!("Expected ", "LIST", " SUCC data type, got ", self.name())?;
		}
		Ok(self.get_list().unwrap())
	}
	
	pub fn expect_string(&self) -> EhResult<&str> {
		if !self.is_value() {
			exception!("Expected ", "VALUE", " SUCC data type, got ", self.name())?;
		}
		Ok(self.get_value().unwrap())
	}
	
	pub fn expect_bool(&self) -> EhResult<bool> {
		let value = self.expect_string().wrap(ex!("While expecting bool"))?;
		Ok(match value {
			"true" | "on" | "yes" | "y" => true,
			"false" | "off" | "no" | "n" => false,
			_ => {
				exception!("Expected ", "boolean", " value, but got: ", value)?
			}
		})
	}
	
	pub fn expect_double(&self) -> EhResult<f64> {
		let value = self.expect_string().wrap(ex!("While expecting double"))?;
		Ok(value.parse::<f64>().map_ex(ex!("Expected floating point number, got: ", value))?)
	}
	
	pub fn expect_color(&self) -> EhResult<Color24> {
		let value = self.expect_string().wrap(ex!("While expecting color"))?;
		if value.len() != 6 {
			exception!("Color code must be exactly 6 characters long, got: '", value, "'")?;
		}
		if value.chars().position(|c| {
			!(c >= '0' && c <= '9' || c >= 'A' && c <= 'F')
		}).is_some() {
			exception!("Color code may only consist of letters ", "0-9A-F", ", got: '", value, "'")?;
		}
		//Data validated, time to parse it:
		Ok(Color24 {
			r: u8::from_str_radix(&value[0..1], 16).unwrap(),
			g: u8::from_str_radix(&value[2..3], 16).unwrap(),
			b: u8::from_str_radix(&value[4..5], 16).unwrap(),
		})
	}
	
	pub fn expect_unsigned(&self) -> EhResult<u32> {
		let value = self.expect_string().wrap(ex!("While expecting unsigned"))?;
		Ok(value.parse::<u32>().map_ex(ex!("Failed to parse unsigned expected number"))?)
	}
	
	pub fn expect_component_address(&self) -> EhResult<u32> {
		let mut value = self.expect_string().wrap(ex!("While expecting component address"))?;
		if !value.starts_with("C-") {
			exception!("Expected component address to start with '", "C-", "', but got: ", value)?;
		}
		value = &value[2..];
		Ok(value.parse::<u32>().map_ex(ex!("Failed to parse component address number"))?)
	}
}
