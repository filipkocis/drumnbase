use std::fs;

use crate::{parser::Schema, file::data::LoadMode, basics::column::Column};

pub trait Parser {
    fn parse(input: &str) -> Result<Schema, String>;
    fn parse_file(file: &str) -> Result<Schema, String>;
}

pub struct SimpleParser;

impl SimpleParser {
    fn parse_new_column(column_name: &str, column_type: &str) -> Result<Column, String> {
        

        todo!()
    }
    fn handle_table_column(schema: &mut Schema, args: &Vec<String>, args_parts: &[&str]) -> Result<(), String> {

        let table_name = args_parts[0];
        let prop = args[2].as_str();
        let column_name = args_parts[3];
        let column_type = args[4].as_str();
        let column_type_parts = args[4].as_str();

        let table = schema.get_table(table_name)
            .ok_or(format!("Table not found: {}", table_name))?;

        match prop {
            "remove" => table.columns.retain(|c| c.name != column_name),
            "add" => {
                let column = Self::parse_new_column(column_name, column_type)?;
                table.columns.push(column)
            }

            _ => return Err(format!("Unknown column command: {}", prop))
        }

        Ok(())
    }

    fn handle_table_set(schema: &mut Schema, args: &Vec<String>, args_parts: &[&str]) -> Result<(), String> {
        if args.len() != 4 { return Err("Invalid table set command arguments".to_string()) }

        let table_name = args_parts[0];
        let prop = args[2].as_str();
        let value_parts = args_parts[3];
        let value = args[3].as_str();

        let table = schema.get_table(table_name)
            .ok_or(format!("Table not found: {}", table_name))?;

        match prop {
            "read_only" => table.read_only = value == "true",
            "load_mode" => table.data.load_mode = match value {
                "memory" => LoadMode::Memory,
                "disk" =>  LoadMode::Disk,
                _ => return Err(format!("Invalid load mode {:?} for table {:?}", value_parts, table_name))
            },

            _ => return Err(format!("Unknown table set property: {}", prop))
        }

        Ok(())
    }

    fn handle_table(schema: &mut Schema, args: &Vec<String>, args_parts: &[&str]) -> Result<(), String> {
        if args.len() < 2 { return Err("Invalid table command arguments".to_string()) }

        let table_name = args_parts[0];
        match args[1].as_str() {
            "create" => schema.add_table(table_name),
            "delete" => schema.delete_table(table_name),
            "set" => Self::handle_table_set(schema, args, args_parts)?,
            "column" => Self::handle_table_column(schema, args, args_parts)?,

            _ => return Err(format!("Unknown table command: {}", args_parts[1]))
        }

        Ok(())
    }

    fn handle_root_dir(schema: &mut Schema, args: &Vec<String>, args_parts:  &[&str]) -> Result<(), String> {
        if args.len() != 1 { return Err("root_dir value not provided".to_string()) }
        schema.root_dir = args_parts[0].to_string();

        Ok(())
    }
}

impl Parser for SimpleParser {
    fn parse(input: &str) -> Result<Schema, String> {
        let mut schema = Schema::default(); 

        for line in input.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 0 { continue }

            let command = String::from(parts[0]).to_lowercase();
            let args_parts = &parts[1..];
            let args: Vec<String> = args_parts.iter().map(|s| s.to_string().to_lowercase()).collect();



            match command.as_str() {
                "root_dir" => Self::handle_root_dir(&mut schema, &args, args_parts)?,
                "table" => Self::handle_table(&mut schema, &args, args_parts)?,

                _ => return Err(format!("Unknown command: {}", command))
            }

        }
        
        todo!()
    }

    fn parse_file(file: &str) -> Result<Schema, String> {
        if let Ok(str) = fs::read_to_string(file) {
            return Self::parse(&str);
        } else {
            return Err("Could not read file".to_string())
        }
    }
}
