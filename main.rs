// nspicer@lutherstem.com
// fast inventory lookup and analysis tool. quickly filters items by inventory,
// quickly search for part numbers within all snapshots of inventory.



use std::io;
use std::string::String;
use calamine::{Reader, Xlsx, open_workbook, DataType};
use calamine::DataType::{String as CString};
use calamine::DataType::{Int as CInt};
use calamine::DataType::{Float as CFloat};
use glob::glob;


// ------------MAIN MENU------------
fn main() {	
	let mut m: Menu = Menu::new();

	while m.running {	// main menu loop
		m.take_input();
	}
}



// MAIN MENU LOGIC
struct Menu {
	running: bool,
	history: History,
}
impl Menu {
	fn new() -> Menu {
		let running: bool = true;
		let history: History = History::new();
		Menu {
			running,
			history,
		}
	}

	// every cycle of the main loop, get input
	fn take_input(&mut self) {
		self.show();
		let mut input = String::new();
        println!("Input...");
        io::stdin().read_line(&mut input).expect("Failed to read line");
        input = input.to_lowercase();

        // MENU OPTIONS
        if input.trim() == "quit" { // stop running tool
        	self.running = false;
        }

        if input.trim() == "find" {
        	self.history.find_items();
        }

        if input.trim() == "lookin" {
       		self.history.look_in();
        }

        if input.trim() == "search" {
        	self.history.search_parts();
        }

        if input.trim() == "list" { // list all inventories (better than lookin for seeing overview)
        	self.history.show();
        }

        if input.trim() == "movers" {
        	self.history.frequent_movers();
        } 

        if input.trim() == "compare" { // figure out what one inventory has that another doesn't. given A, it sees if its parts are in B. if A is not in B, add to that list.
        	self.history.compare_inv()
        }

        if input.trim() == "cls" {
        	print!("\x1B[2J");
        }

        

	}

	fn show(&self) {
		println!("\nHISTORIC OPTIONS\n---------------------\nFIND   (by description)\nSEARCH (by part number)\nLOOKIN\nLIST\nCOMPARE\nQUIT\n");
	}
}



// ----------META DATA-----------
struct History {
	inventories: Vec<Inventory>,
}
impl History {
	fn new() -> History {

		let mut input = String::new();
		println!("History of 2022, 2023, or 2025?....");
        io::stdin().read_line(&mut input).expect("Failed to read line");
        input = input.to_lowercase();

		let inventories = History::get_inventories(input); // search for excel workbooks in some directory
		History {
			inventories,
		}
	}


	fn compare_inv(&mut self) { // this should be refactored
		
		let mut input1 = String::new();
        println!("Index of Inventory A");
        io::stdin().read_line(&mut input1).expect("Failed to read line");
        let input1 = input1.trim().parse::<usize>().expect("Invalid");

        let mut input2 = String::new();
        println!("Index of Inventory B");
        io::stdin().read_line(&mut input2).expect("Failed to read line");
		let input2 = input2.trim().parse::<usize>().expect("Invalid");

		let a: &Inventory = &self.inventories[input1];
		let b: &Inventory = &self.inventories[input2];

		let mut temp: Inventory = Inventory::new(&String::from("compared".to_owned() + &input1.to_string() + "_" + &input2.to_string()));

		
		for item in a.items.iter() { // IS ITEM IN INV A IS NOT IN INV B, ADD THAT TO THE COMPARED INVENTORY
			if b.item_absent(item) {
				temp.items.push(Item::new( (*item.part_number).to_string(), item.quantity, (*item.description).to_string(), item.price));
			}
		}



		 //only if it needs to be a bi-directional comparison. unique items from BOTH inventories.

		/*for item in b.items.iter() {

			if a.item_absent(item) && temp.item_absent(item){
				temp.items.push(Item::new( (*item.part_number).to_string(), item.quantity, (*item.description).to_string(), item.price));
			}

		}*/
		
/*
		for item in a.items.iter() {
			for item2 in b.items.iter() {
				if item.eq(item2) {
					temp.items.push(Item::new( (*item.part_number).to_string(), item.quantity - item2.quantity, (*item.description).to_string(), item.price))
				}
			}
		} 
		*/

		self.inventories.push(temp);

	}



	// -----STATS-----
	fn frequent_movers(&self) { // TODO TODO TODO TODO 
		println!("--------------------------------------------------------------");

	}

	fn search_parts(&self) {
		let mut input = String::new();
		println!("--------------------------------------------------------------");
		println!("Enter Part(ial) Number");
		io::stdin().read_line(&mut input).expect("failed to read line");
		let mut temp_inv: Inventory = Inventory::new(&input);
		for inventory in self.inventories.iter() {
			for item in inventory.items.iter() {
				if item.part_number.to_uppercase().trim().contains(&input.trim().to_uppercase()) {
					// add item if it is not in temp, otherwise add that items quantity to temp_inv
					if temp_inv.item_absent(item) {
						temp_inv.items.push(Item::new( (*item.part_number).to_string(), item.quantity, (*item.description).to_string(), item.price));
						//println!("added {} {}", item.part_number, inventory.name);
					}
					else {
						let index = temp_inv.items.iter().position(|i| i == item).unwrap();
						temp_inv.items[index].quantity += item.quantity;
					}					
				}
			}
		}
		temp_inv.items.sort_unstable_by_key(|item| item.quantity);
		println!();
		for item in temp_inv.items.iter() {
			item.show();
		}
		println!("--------------------------------------------------------------");
	}
	


	fn look_in(&self) {
		let mut input = String::new();
		println!("--------------------------------------------------------------");
		self.show();
		println!("Enter Inventory Number");
		io::stdin().read_line(&mut input).expect("failed to read line");
		let input = input.trim().parse::<usize>().expect("Invalid");
		if input < self.inventories.len() {
			self.inventories[input].inventory_menu();
		}
		else {
			println!("Invalid, Try Again");
			self.look_in();
		}
		println!("--------------------------------------------------------------");
	}

	fn find_items(&self) { // counts up items with a certain keyword in their description and gives back a sum total of the quantities over the inventory snapshots
		let mut input = String::new();
		println!("-------------------------------------------------------------- (QUANTITIES ARE ADDITIVE OVER TIME! NOT REPRESENTATIVE OF CURRENT QUANTITIES!)");
		println!("Enter Keyword");
		io::stdin().read_line(&mut input).expect("failed to read line");
		let mut temp_inv: Inventory = Inventory::new(&input);
		for inventory in self.inventories.iter() {
			for item in inventory.items.iter() {
				if item.description.to_uppercase().trim().contains(&input.trim().to_uppercase()) {
					// add item if it is not in temp, otherwise add that items quantity to temp_inv
					if temp_inv.item_absent(item) {
						temp_inv.items.push(Item::new( (*item.part_number).to_string(), item.quantity, (*item.description).to_string(), item.price));
						//println!("added {} {}", item.part_number, inventory.name);
					}
					else {
						let index = temp_inv.items.iter().position(|i| i == item).unwrap();
						temp_inv.items[index].quantity += item.quantity;
					}					
				}
			}
		}
		temp_inv.items.sort_unstable_by_key(|item| item.quantity);
		println!();
		for item in temp_inv.items.iter() {
			item.show();
		}
		println!("--------------------------------------------------------------");
	}


	// -----HELPER FUNCTIONS-----
	fn show(&self) {
		for (c, i) in (0_u8..).zip(self.inventories.iter()) {
			print!("{} - ", c);
			i.show();
		}
	}

	fn get_inventories(year: String) -> Vec<Inventory> { // grabs all workbooks from a given file path 
		let mut inventories = Vec::new();
		for entry in glob("**/*.xlsx").expect("failed to read pattern") { // looks for xlsx files in the current working directory.
			match entry {
				Ok(path) => { // path is good, get path and send it to file reader to create inventory
					let pth = format!("\\Users\\Nick\\projects\\inventory_tool\\{}", path.display());
					if pth.contains(year.trim()) {
						inventories.push(Inventory::read_file(pth)); // inventory from excel sheet
					}
				},
				Err(e) => println!("{:?}", e), // oh no
			}
		}
		inventories
	}
}



// ----------INVENTORY REPRESENTATION----------
struct Inventory {
	items: Vec<Item>,
	name: String
}

impl PartialEq for Inventory {
	fn eq(&self, other: &Inventory) -> bool {
		self.name == other.name
	}
}

impl Inventory {
	fn new(name: &String) -> Inventory {
		Inventory {
			items: Vec::new(),
			name: name.to_string(),
		}
	}


	// -----STATS-----
	fn pattern_search(&self) {  // search part numbers by pattern matching
		let mut input = String::new();
		println!("****************************************************");
		println!("Enter a Pattern (example: xxx-xxx-xxxx)");
		io::stdin().read_line(&mut input).expect("failed to read line");
		let mut temp_inv: Inventory = Inventory::new(&input);
		let mut hyphens = Vec::new();
		let length: u8 = input.trim().len() as u8;
		// hyphens in patters
		for (i, c) in input.trim().chars().enumerate() {
			if c == '-' {
				hyphens.push(i);
			}
		}
		// matching
		for item in self.items.iter() {
			let mut hb = true;
			let mut hc = 0;
			for c in item.part_number.chars() {
				if c == '-' {
					hc += 1;
				}
			}
			if item.part_number.len() as u8 == length && hc == hyphens.len() { // match criteria
				for h in hyphens.iter() {
					//println!("{} : {}", &item.part_number, &item.part_number[*h..*h+1] );
					if item.part_number[*h..*h+1] != String::from("-") {
						hb = false;
					}
				}
				if hb { // if correct add to temp
					temp_inv.items.push(Item::new((*item.part_number).to_string(), item.quantity, (*item.description).to_string(), item.price));				
				}
			}	
		}
		// sort by value and show
		temp_inv.items.sort_unstable_by_key(|item| item.quantity*(item.price as i32));
		println!("\n\n\n****************************************************");
		println!("Pattern: {}****************************************************", input);
		for item in temp_inv.items.iter() {
			item.show();
		}
		println!("****************************************************")
	}


	fn filter_parts(&self) { // select a filter parameter and two values for the filter to stay between
		// price, value, quantity & above/below threshold values.
		let mut temp_inv = Inventory::new(&"Filtered".to_string());
		let mut input = String::new();
		println!("****************************************************");
		println!("Filter Options\n************\nPRICE\nVALUE\nAMOUNT\n\nInput...");
		io::stdin().read_line(&mut input).expect("failed to read line");
		let filter = input.trim().to_lowercase();
		let mut input = String::new();
		println!("\nEnter Lower Bound");
		io::stdin().read_line(&mut input).expect("failed to read line");
		let lower_bound: i32 = input.trim().parse::<i32>().expect("failed lower");
		let mut input = String::new();
		println!("\nEnter Upper Bound");
		io::stdin().read_line(&mut input).expect("failed");
		let upper_bound: i32 = input.trim().parse::<i32>().expect("failed upper");
		if filter == "price" {
			for item in self.items.iter() {
				if item.price >= lower_bound as f32 && item.price <= upper_bound as f32 {
					temp_inv.items.push(Item::new( (*item.part_number).to_string(), item.quantity, (*item.description).to_string(), item.price));
				}
			}
			temp_inv.items.sort_unstable_by_key(|item| item.price as i32);
		}
		if filter == "value" {
			for item in self.items.iter() {
				if (item.price as i32)*item.quantity >= lower_bound && (item.price as i32)*item.quantity <= upper_bound {
					temp_inv.items.push(Item::new( (*item.part_number).to_string(), item.quantity, (*item.description).to_string(), item.price));
				}
			}
			temp_inv.items.sort_unstable_by_key(|item| item.quantity*(item.price as i32));

		}
		if filter == "amount" {
			for item in self.items.iter() {
				if item.quantity >= lower_bound && item.quantity <= upper_bound {
					temp_inv.items.push(Item::new( (*item.part_number).to_string(), item.quantity, (*item.description).to_string(), item.price));
				}
			}
			temp_inv.items.sort_unstable_by_key(|item| item.quantity);
		}
		println!("\n\n\n*******************************************************************");
		println!("Filter: {}   Lower Bound: {}   Upper Bound: {}", filter, lower_bound, upper_bound);
		println!("*******************************************************************");
		for item in temp_inv.items.iter() {
			item.show();
		}

		let mut sum:i32 = 0;
		for item in temp_inv.items.iter() {
			sum += item.price as i32 * item.quantity;
		}
		println!("Sum: {}", sum);
		println!("*******************************************************************\n\n");
		
	}

	fn part_lookup(&self) {
		let mut input = String::new();
		println!("Enter Part Number");
		io::stdin().read_line(&mut input).expect("Failed to read line");
		for item in self.items.iter() {
			if item.part_number.to_uppercase() == input.trim().to_uppercase() {
				println!("\n***************************");
				item.show();
				println!("Total Value: ${}", item.get_total_value());
				println!("***************************");
				return;
			}
		}
		println!("Item not found.");
	}

	fn find_parts(&self) {
		let mut input = String::new();
		println!("***************************");
		println!("Enter Keyword");		
		io::stdin().read_line(&mut input).expect("Failed to read line");
		let mut temp_inv = Inventory::new(&input);
		for item in self.items.iter() {
			if item.description.to_uppercase().trim().contains(input.to_uppercase().trim()) && temp_inv.item_absent(item) {
				temp_inv.items.push(Item::new( (*item.part_number).to_string(), item.quantity, (*item.description).to_string(), item.price));	
			}
		}
		temp_inv.items.sort_unstable_by_key(|item| item.quantity * (item.price as i32));
		println!();
		for item in temp_inv.items.iter() {
			item.show();
		}
		println!("***************************");
	}

	fn get_total_value(&self) -> f32 { // sum of the prices of every single item in inventory, including quantity
		let mut  sum: f32 = 0.0;
		for item in self.items.iter() {
			sum += item.get_total_value();
		}
		sum
	}

	fn search_parts(&self) {
		let mut input = String::new();
		println!("**************************************************************");
		println!("Enter Part(ial) Number");
		io::stdin().read_line(&mut input).expect("failed to read line");
		let mut temp_inv: Inventory = Inventory::new(&input);
		for item in self.items.iter() {
			if item.part_number.to_uppercase().trim().contains(&input.trim().to_uppercase()) {
				// add item if it is not in temp, otherwise add that items quantity to temp_inv
				if temp_inv.item_absent(item) {
					temp_inv.items.push(Item::new( (*item.part_number).to_string(), item.quantity, (*item.description).to_string(), item.price));
					//println!("added {} {}", item.part_number, inventory.name);
				}
				else {
					let index = temp_inv.items.iter().position(|i| i == item).unwrap();
					temp_inv.items[index].quantity += item.quantity;
				}					
			}
		}
		temp_inv.items.sort_unstable_by_key(|item| item.quantity);
		println!();
		for item in temp_inv.items.iter() {
			item.show();
		}
		println!("**************************************************************");
	}


	// -----HELPER FUNCTIONS-----
	fn inventory_menu(&self) {
		println!();
		loop {
			let mut input = String::new();
			self.show_menu();
			println!("\nInput...");
			io::stdin().read_line(&mut input).expect("failed to read line");
			if input.trim().to_lowercase() == "quit" {
				break;
			}
			if input.trim().to_lowercase() == "part" {
				self.part_lookup();
			}
			if input.trim().to_lowercase() == "find" {
				self.find_parts();
			}
			if input.trim().to_lowercase() == "filter" {
				self.filter_parts();
			}
			if input.trim().to_lowercase() == "search" {
				self.search_parts();
			}
			if input.trim().to_lowercase() == "match" {
				self.pattern_search();
			}
			if input.trim() == "cls" {
        		print!("\x1B[2J");
        	}
		}
	}

	fn show_menu(&self) {
		println!("\nInventory Options - {}\n***************\nPART\nFIND   (by description)\nSEARCH (by part number)\nFILTER\nMATCH\nQUIT", self.name);
	}

	fn show(&self) {
		println!("{}: total value - ${}", self.name, self.get_total_value());
	}

	fn add_item(&mut self, item: Item) { // check if an item is not in this inventory, adds it if so.
		if self.item_absent(&item) {
			self.items.push(item);
		}
	}

	fn item_absent(&self, item: &Item) -> bool { // checks  if an item is present
		for i in self.items.iter() {
			if i == item {
				return false;
			}
		}
		true
	}

	fn read_file(path: String) -> Inventory {
		let mut excel: Xlsx<_> = open_workbook(&path).unwrap(); // PAY ATTENTION TO THIS FUCKING THING -> if error, close Excel Sheet
		let mut inventory = Inventory::new(&path);
		println!("{}", path);
		if let Some(Ok(r)) = excel.worksheet_range("ExcelExportTmp") { // whatever the name of the workbook is at the bottom of the excel file
	    	for row in r.rows() {
	    		if row[0].to_string() == "Part Number" { continue; } // skip first row ::: THIS MEANS HAVE 'Part Number' WRITTEN IN THE TOP LEFT CELL OF THE EXCEL SHEET
	    		else {

	    				//USE THESE TO MATCH DATA FROM EXCEL COLUMNS. ROW INDEX MUST MATCH COLUMN.
	    			let pn: String = row[3].to_string();
		        	let qty: i32   = Item::qty_from_string(&row[6]);
		        	let desc: String = row[4].to_string();
		        	let price: f32 = Item::price_from_string(&row[7]);

		        	inventory.add_item(Item::new(pn, qty, desc, price)); 
		        }
	    	}

		}
		inventory
	}
}



// -----------ITEM REPRESENTATION-----------
struct Item {
	part_number: String,
	quantity: i32,
	description: String,
	price: f32
}

impl PartialEq for Item {
	fn eq(&self, other: &Item) -> bool {
		self.part_number.to_uppercase() == other.part_number.to_uppercase() && self.description == other.description
	}
}

impl Item {
	fn new(part_number: String, quantity: i32, description: String, price: f32) -> Item {
		Item {
			part_number,
			quantity,
			description,
			price
		}
	} 

	// -----STATS-----
	fn get_total_value(&self) -> f32 {
		if self.quantity > 0 { self.quantity as f32 * self.price }
		else { 0. } // if negative, just return 0. dont need negatives
	}

	// -----HELPER FUNCTIONS-----
	fn show(&self) { // print out item info
		println!(
			"PN: {}\nDescription: {}\nQty: {}\nPrice: {}\n", self.part_number, self.description, self.quantity, self.price);
	}

	fn qty_from_string(qty: &DataType) -> i32 { // turns excel cell data numbers(text, float, int) into rust numbers(i32)
		fn qfs_decindex(s: &String) -> usize {
			let bytes = s.as_bytes();
			for (i, &item) in bytes.iter().enumerate() {
				if item == b'.' {
					return i;
				}
			}
			panic!("empty cell! ctrl f for DEBUG HELP or ERROR");
		}
		match qty {
			CString(s) => {
				println!("{:?}",s );     // FOR ULTRA messed UP DEBUG PURPOSES KEYWORDS TO FIND THIS LINE LATER:DEBUG HELP DEBUG HELP FUCKED BROKEN SHIT FUCK ERROR MAJOR BAD MISTAKE
				
				// do some conditional shit for when we come across a string with a decimal in it
				{
					if s.contains(".") {
						let i = qfs_decindex(s);
						//panic!("oh no! a decimal in a string!"); 
						let s2 = &s[..i].trim();
						println!("{:?} has a decimal. fix this in RB.", s2 );
						//panic!("good stop");
						return s2.parse::<i32>().unwrap(); // sneaky s2 return! mind you!
					}
				}
				 s.parse().unwrap()
			},

			CInt(s) => { // Integer case  (in case of manual edits to excel file)
				*s as i32
			},

			CFloat(s) => { // Float case (just-in-case-somehow)
				//println!("how did you get a float here bro");
				//println!("{:?}", s);
				s.round().rem_euclid(2f64.powi(32)) as u32 as i32 // joke of the year 2023 (posted to this code 10/10 10:31PM) under no circumstances should we end up here in quantity calcs, BUT WE DO!
			},

			_ => {
				panic!("oh no, our table! THIS IS LIKE, BAD, BAD! THIS IS THE END CASE AFTER EVERYTHING ELSE FAILS NOW!");
			}
		}
	}

	fn price_from_string(price: &DataType) -> f32 { // excel cell price to rust float
		match price {
			CString(s) => { // extract float from price
				println!("{:?}", s);
				s[0..].parse().unwrap()  // maybe get rid 1 index slice for things without dollar sign
			},
			CFloat(s) => {
				*s as f32
			}
			_ => 1.0
		}
	}
}







// ----------------------------------------------------NEW FEATURES---------------------------------------------------









// ---------- TRACKER ----------- NOTIFIES USER WHEN CERTAIN ITEMS ARE BELOW A CERTAIN LEVEL AFTER A SNAPSHOT. OUGHT TO BE ACCESSIBLE FROM THE MAIN MENU.
/*
struct Tracker {

}
impl Tracker {
	fn new() -> Tracker {
		Tracker {

		}
	}
}
*/