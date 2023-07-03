use crate::prelude::*;
use crate::util::ansi_constants::ansi_reset;
use crate::util::log_formatter::{color_debug_normal, color_debug_highlight};

use std::{str::from_utf8, collections::HashMap};
use std::cell::{RefCell, RefMut};
use std::cmp::Ordering;
use std::rc::Rc;

pub enum SuccType {
	Any(), //Could be literally any of the below types, but always a length of zero
	Value(String), //Contains a single text value
	Map(HashMap<String, SuccType>), //Contains a dictionary
	List(Vec<SuccType>), //Contains a list
}

pub fn debug_succ_file(bytes: &[u8]) -> EhResult<()> {
	let root_dict = parse_succ_file(bytes)?;
	debug_print(&root_dict);
	Ok(())
}

pub fn debug_print(entry: &SuccType) {
	print_inner(entry,
		concat!(color_debug_normal!(), "└ ").to_owned(),
		concat!(color_debug_normal!(), "  ").to_owned(),
	);
}

fn print_inner(entry: &SuccType, entry_prefix: String, prefix: String) {
	match entry {
		SuccType::Any() => {
			println!(concat!("{}", color_debug_highlight!(), "{}", ansi_reset!()), entry_prefix, "---");
		}
		SuccType::Value(value) => {
			println!(concat!("{}'", color_debug_highlight!(), "{}", color_debug_normal!(), "'", ansi_reset!()), entry_prefix, value);
		}
		SuccType::Map(map) => {
			println!(concat!("{}<map>", ansi_reset!()), entry_prefix);
			for (index, (key, value)) in map.iter().enumerate() {
				print_inner(value,
					format!(concat!(
						"{}{} ", color_debug_highlight!(), "{}", color_debug_normal!(), ": "),
						 prefix, if index == (map.len() - 1) { '└' } else { '├' }, key
					),
					format!("{}{} ",
						 prefix, if index == (map.len() - 1) { ' ' } else { '│' }
					),
				);
			}
		}
		SuccType::List(list) => {
			println!(concat!(color_debug_normal!(), "{}<list>", ansi_reset!()), entry_prefix);
			for (index, value) in list.iter().enumerate() {
				print_inner(value,
					format!(concat!(
						"{}{} "),
						prefix, if index == (list.len() - 1) { '└' } else { '├' }
					),
					format!("{}{} ",
						 prefix, if index == (list.len() - 1) { ' ' } else { '│' }
					),
				);
			}
		}
	}
}

#[derive(Eq, PartialEq)]
#[derive(Debug)]
enum SuccTypeInner {
	Any,
	Value,
	Map,
	List,
}

type Type = Rc<RefCell<LineContext>>;

struct TreeParser {
	root_elements: Vec<Type>,
	stack: Vec<Type>,
}

impl TreeParser {
	fn new() -> Self {
		Self {
			root_elements: Vec::new(),
			stack: Vec::new(),
		}
	}
	
	fn has_no_parent(&self) -> bool {
		self.stack.is_empty()
	}
	
	fn top_entry(&mut self) -> EhResult<RefMut<LineContext>> {
		self.stack.last_mut().unwrap().try_borrow_mut().map_ex(ex!("While getting top of stack, no element present or already locked => the developer writing the succ parsing code messed up."))
	}
	
	fn add_root(&mut self, line_meta: LineMeta) -> EhResult<()> {
		if line_meta.is_list() {
			exception!("Root level SUCC entries need a key, they may not be list entries")?;
		}
		let line = Rc::new(RefCell::new(LineContext::new(line_meta)));
		let copy = line.clone();
		self.root_elements.push(line);
		self.stack.push(copy);
		Ok(())
	}
}

#[derive(Debug)] //Just so, that unwrap/expect can be used on it... Thanks Rust.
struct LineContext {
	meta: LineMeta,
	children: Vec<Type>,
	expected_child_indentation: usize,
	determined_type: SuccTypeInner,
}

impl LineContext {
	fn new(meta: LineMeta) -> Self {
		Self {
			determined_type: if meta.is_parent() { SuccTypeInner::Any } else { SuccTypeInner::Value },
			meta,
			children: Vec::new(),
			expected_child_indentation: 0,
		}
	}
}

#[derive(Debug)] //Cause I am forced to...
struct LineMeta {
	indentation: usize,
	key: Option<String>,
	value: Option<String>,
}

impl LineMeta {
	fn is_list(&self) -> bool {
		self.key.is_none()
	}
	
	fn get_data_type(&self) -> SuccTypeInner {
		if self.is_list() {
			SuccTypeInner::List
		} else {
			SuccTypeInner::Map
		}
	}
	
	fn is_parent(&self) -> bool {
		self.value.is_none()
	}
}

fn parse_line(line: &str) -> EhResult<Option<LineMeta>> {
	//Helper for parsing stages:
	enum Stage {
		Indentation,
		Key,
		BeforeValue,
		Value
	}
	let mut stage = Stage::Indentation;
	let mut is_escaping = false;
	let mut trim_hack = 0;
	//Output values:
	let mut key_builder = String::new();
	let mut value_builder = String::new();
	let mut indentation = 0;
	for c in line.chars() {
		if let Stage::Indentation = stage {
			//Reading the spaces at the beginning of a line, to get the indentation level:
			if c == ' ' {
				indentation += 1;
				continue;
			} else if c == '#' {
				return Ok(None); //This line only contains a comment.
			} else if c == ':' {
				exception!("Corrupted SUCC line, key may not start with a colon: '", line, "'")?;
			} else if c == '-' {
				stage = Stage::BeforeValue;
				continue;
			} else {
				//Whatever character comes here, it must be part of the key.
				stage = Stage::Key;
			}
		}
		if let Stage::Key = stage {
			//Reading the key of line:
			if c == ':' {
				trim_hack = 0; //Reset
				stage = Stage::BeforeValue;
				continue;
			} else if c == '#' {
				exception!("Corrupted SUCC line, key may not contain a # character: '", line, "'")?;
			} else if c == ' ' {
				//Apply spaces later, once another letter has been found:
				trim_hack += 1;
				continue;
			} else {
				//Append skipped space:
				if trim_hack != 0 {
					for _ in 0..trim_hack {
						key_builder.push(' ');
					}
					trim_hack = 0;
				}
				//Append actual character:
				key_builder.push(c);
			}
		}
		if let Stage::BeforeValue = stage {
			//Skip spaces until a value start has been found:
			if c == ' ' {
				continue; //Not relevant right now.
			} else if c == '#' {
				break; //Only a comment here, no value.
			} else {
				//We got a line start!
				stage = Stage::Value;
			}
		}
		if let Stage::Value = stage {
			//Actually reading the value now:
			if c == ' ' {
				if is_escaping {
					value_builder.push('\\');
					is_escaping = false;
				}
				trim_hack += 1;
			} else if c == '\\' {
				if is_escaping {
					value_builder.push('\\');
				} else {
					//Append skipped space:
					if trim_hack != 0 {
						for _ in 0..trim_hack {
							value_builder.push(' ');
						}
						trim_hack = 0;
					}
					is_escaping = true;
				}
			} else if c == '#' {
				if is_escaping {
					value_builder.push('#');
					is_escaping = false;
				} else {
					break; //Reached end of content, rest is comment.
				}
			} else {
				//Append skipped escaping:
				if is_escaping {
					value_builder.push('\\');
					is_escaping = false;
				}
				//Append skipped space:
				if trim_hack != 0 {
					for _ in 0..trim_hack {
						value_builder.push(' ');
					}
					trim_hack = 0;
				}
				//Append normal data:
				value_builder.push(c);
			}
		}
	}
	if let Stage::Indentation = stage {
		return Ok(None); //Line is empty.
	}
	if let Stage::Key = stage {
		exception!("Corrupted SUCC line, key started but not ended: '", line, "'")?;
	}
	if let Stage::Value = stage {
		if is_escaping {
			value_builder.push('\\');
		}
	}
	Ok(Some(LineMeta {
		indentation,
		key: if key_builder.is_empty() { None } else { Some(key_builder) },
		value: if value_builder.is_empty() { None } else { Some(value_builder) },
	}))
}

//Helper macro to not copy paste these 5 lines dozen of times. Cannot really put them in a function, due to specific borrowing constrains.
macro_rules! add_child {
	($stack:expr, $top_entry:expr, $line_meta:expr) => {
		let line = Rc::new(RefCell::new(LineContext::new($line_meta)));
		let copy = line.clone();
		$top_entry.children.push(line);
		std::mem::drop($top_entry); //Drop it here, to make borrow checker be quiet about another usage of stack.
		$stack.push(copy);
		if $stack.len() > 200 {
			exception!("SUCC nesting level of above 200 is currently for safety not allowed. Complain to project maintainer.")?;
		}
	};
}

pub fn parse_succ_file(bytes: &[u8]) -> EhResult<SuccType> {
	let text = from_utf8(bytes).map_ex(ex!())?; //TBI: Should this be read as string by default?
	let lines = text.lines();
	
	let mut data = TreeParser::new();
	
	for line in lines {
		let line_meta = parse_line(line).wrap(ex!("While parsing line content"))?;
		if line_meta.is_none() {
			continue;
		}
		let line_meta = line_meta.unwrap();
		
		//Process line:
		if data.has_no_parent() {
			//Stack is empty, means we have a root element, that needs indentation 0:
			if line_meta.indentation != 0 {
				exception!("First SUCC data line needs indentation 0")?;
			}
			data.add_root(line_meta)?;
			continue;
		}
		
		let mut last_line = data.top_entry()?;
		match line_meta.indentation.cmp(&last_line.meta.indentation) {
			Ordering::Greater => {
				//New first child entry!
				if last_line.determined_type != SuccTypeInner::Any {
					exception!("Cannot add child entry, if the parent has a value set!")?;
				}
				last_line.determined_type = if line_meta.is_list() { SuccTypeInner::List } else { SuccTypeInner::Map };
				last_line.expected_child_indentation = line_meta.indentation;
				add_child!(data.stack, last_line, line_meta);
			}
			Ordering::Equal => {
				//Sibling entry:
				std::mem::drop(last_line); //Yeah I really want to use the stack again...
				data.stack.pop(); //Get rid of the previous sibling, to get the parent.
				//Get next parent:
				if data.has_no_parent() {
					//Got another root element:
					if line_meta.indentation != 0 {
						exception!("Got no parent, but the indentation was not 0? Implementation faulty?")?;
					}
					data.add_root(line_meta)?;
				} else {
					//Add new sibling to parent:
					let mut last_line = data.top_entry()?;
					if last_line.determined_type != line_meta.get_data_type() {
						exception!("Cannot mix list and dict collection entries with the same parent.")?;
					}
					add_child!(data.stack, last_line, line_meta);
				}
			}
			Ordering::Less => {
				loop {
					//Some parent's sibling entry:
					std::mem::drop(last_line); //Yeah I really want to use the stack again...
					data.stack.pop(); //Get rid of this parent as its not relevant anymore.
					//Get next parent:
					if data.has_no_parent() {
						//Got another root element:
						if line_meta.indentation != 0 {
							exception!("Wrongly indented SUCC entry! Expected indentation ", 0, " but got ", line_meta.indentation)?;
						}
						data.add_root(line_meta)?;
						break; //Exit "try-again" loop.
					} else {
						//Add new sibling to parent:
						last_line = data.top_entry()?;
						if line_meta.indentation > last_line.expected_child_indentation {
							//Parent entry has even less indentation, meaning the child is somewhere between parent entries...
							exception!("Wrongly indented SUCC entry! Expected indentation ", last_line.expected_child_indentation, " but got ", line_meta.indentation)?;
						}
						if line_meta.indentation == last_line.expected_child_indentation {
							//Found the correct parent entry!
							if last_line.determined_type != line_meta.get_data_type() {
								exception!("Cannot mix list and dict collection entries with the same parent.")?;
							}
							add_child!(data.stack, last_line, line_meta);
							break;
						}
						//Did not find the correct parent, will keep trying.
					}
				}
			}
		}
	}
	data.stack.clear(); //Get rid of content.
	
	let mut output_types = HashMap::new();
	let root_nodes = data.root_elements;
	
	for node in root_nodes.into_iter() {
		let mut unwrapped = Rc::try_unwrap(node).expect("Something went wrong. Unwrapping RC should not fail at this point, as nothing uses it anymore!").into_inner();
		let key = unwrapped.meta.key.take().unwrap();
		output_types.insert(key, convert_structure(unwrapped)?);
	}
	
	Ok(SuccType::Map(output_types))
}

fn convert_structure(input: LineContext) -> EhResult<SuccType> {
	Ok(match input.determined_type {
		SuccTypeInner::Any => SuccType::Any(),
		SuccTypeInner::Value => {
			SuccType::Value(input.meta.value.unwrap())
		},
		SuccTypeInner::Map => {
			let mut child_nodes = HashMap::new();
			for node in input.children {
				let mut unwrapped = Rc::try_unwrap(node).map_ex(ex!("Something went wrong. Unwrapping RC should not fail at this point, as nothing uses it anymore!"))?.into_inner();
				let key = unwrapped.meta.key.take().unwrap();
				child_nodes.insert(key, convert_structure(unwrapped)?);
			}
			SuccType::Map(child_nodes)
		}
		SuccTypeInner::List => {
			let mut child_nodes = Vec::new();
			for node in input.children {
				let unwrapped = Rc::try_unwrap(node).map_ex(ex!("Something went wrong. Unwrapping RC should not fail at this point, as nothing uses it anymore!"))?.into_inner();
				child_nodes.push(convert_structure(unwrapped)?);
			}
			SuccType::List(child_nodes)
		}
	})
}
