use std::fs;

use crate::{
    parser::Schema, 
    file::data::LoadMode, 
    basics::{column::{Column, ColumnType, NumericType, TextType}, table::Table, row::Row}, utils::log
};

pub trait Parser {
    fn parse(input: &str) -> Result<Schema, String>;
    fn parse_file(file: &str) -> Result<Schema, String>;
}

pub struct SimpleParser;

impl SimpleParser {
    fn parse_new_column(column_name: &str, column_type_str: &str, args: &[&str]) -> Result<Column, String> {
        let column_type = match column_type_str {
            "int_u8" => ColumnType::Numeric(NumericType::IntU8),
            "int_u16" => ColumnType::Numeric(NumericType::IntU16),
            "int_u32" => ColumnType::Numeric(NumericType::IntU32),
            "int_u64" => ColumnType::Numeric(NumericType::IntU64),

            "int_i8" => ColumnType::Numeric(NumericType::IntI8),
            "int_i16" => ColumnType::Numeric(NumericType::IntI16),
            "int_i32" => ColumnType::Numeric(NumericType::IntI32),
            "int_i64" => ColumnType::Numeric(NumericType::IntI64),

            "float_32" => ColumnType::Numeric(NumericType::Float32),
            "float_64" => ColumnType::Numeric(NumericType::Float64),
            
            "char" => ColumnType::Text(TextType::Char),
            // "variable" => ColumnType::Text(TextType::Variable),
            s if s.starts_with("fixed") => {
                let mut fixed_length = 255;

                if s.starts_with("fixed(") && s.ends_with(")") {
                    if let Some(val) = s.strip_prefix("fixed(").unwrap().strip_suffix(")") {
                        fixed_length = val.parse().unwrap();        
                    } else {
                        return Err(format!("Invalid fixed column type length: {}", s).to_string())
                    }
                } else if s != "fixed" {
                    return Err(format!("Invalid column type syntax: {}", s).to_string())
                }

                ColumnType::Text(TextType::Fixed(fixed_length))
            },

            "bool" | "boolean" => ColumnType::Boolean,

            _ => return Err(format!("unknown column type: {}", column_type_str).to_string())
        };

        // TODO: refactor this, move outside
        let column_length = match column_type {
            ColumnType::Text(TextType::Fixed(len)) => len,
            ColumnType::Text(TextType::Char) => 1,
            ColumnType::Numeric(NumericType::Float32) => 4,
            ColumnType::Numeric(NumericType::IntU32) => 4,
            ColumnType::Boolean => 1,
            _ => todo!("column length for type: {:?}", column_type)
        };

        let mut column = Column::new(column_name, column_type);
        column.length = column_length;

        for &arg in args {
            let parts: Vec<&str> = arg.split("=").collect();

            match parts[0] {
                "default" => {
                    if parts.len() !=2 { return Err("Invalid column length argument".to_string()) }
                    let default_value = parts[1];

                    if let Err(e) = column.validate(default_value) {
                        return Err(format!("Invalid default value. {}", e))
                    }

                    column.default = Some(default_value.to_string());
                },
                "not_null" => column.not_null = true,
                "unique" => column.unique = true,
                "read_only" => column.read_only = true,

                _ => return Err(format!("Unknown new column property {}", arg).to_string())
            }
        }

        Ok(column)
    }

    fn handle_table_row_get(table: &mut Table, args_parts: &[&str]) -> Result<(), String> {

        todo!()
    }

    fn handle_table_row_delete(table: &mut Table, args_parts: &[&str]) -> Result<(), String> {
        
        todo!()
    }

    fn handle_table_row_add(table: &mut Table, args_parts: &[&str]) -> Result<(), String> {
        let required_columns_count = table.columns.iter().filter(|c| c.not_null).count();

        if args_parts.len() < required_columns_count {
            return Err(format!("Invalid row add arguments, expected {} got {}", required_columns_count, args_parts.len()))
        }

        let mut row = Row::new();

        for (i, column) in table.columns.iter().enumerate() {
            let current_arg = *args_parts
                .iter()
                .find(|s| 
                    s.starts_with(format!("{}=", column.name).as_str())
                )
                .ok_or(format!("Missing row field for column: {}", column.name))?;

            let parts: Vec<&str> = current_arg.split("=").collect();
            if parts.len() != 2 { return Err(format!("Invalid row add argument: {}", current_arg)) }

            let value = parts[1];
            let valid_value = column.validate(value)?;   

            row.set(i, valid_value);
        }

        table.data.buf_rows.push(row);
        Ok(())
    }

    /// table <table_name> row <row_command> [row_args...]
    fn handle_table_row(schema: &mut Schema, args: &Vec<String>, args_parts: &[&str]) -> Result<(), String> {
        if args.len() < 4 { return Err("Invalid table row command arguments length".to_string()) }

        let table_name = args_parts[0];
        let command = args[2].as_str();
        let args_parts = &args_parts[3..];

        let table = schema.get_table(table_name)
            .ok_or(format!("Table not found: {}", table_name))?;

        match command {
            "delete" => Self::handle_table_row_delete(table, args_parts)?,
            "add" => Self::handle_table_row_add(table, args_parts)?,
            "get" => Self::handle_table_row_get(table, args_parts)?,

            _ => return Err(format!("Unknown table row command: {}", command))
        }

        Ok(())
    }

    /// table <table_name> column <column_name> <column_command> <column_type> [column_args...]
    fn handle_table_column(schema: &mut Schema, args: &Vec<String>, args_parts: &[&str]) -> Result<(), String> {
        if args.len() < 4 { return Err("Invalid table column command arguments length".to_string()) }

        let table_name = args_parts[0];
        let column_name = args_parts[2];
        let command = args[3].as_str();

        let table = schema.get_table(table_name)
            .ok_or(format!("Table not found: {}", table_name))?;

        match command {
            "delete" => table.columns.retain(|c| c.name != column_name),
            "add" => {
                if args.len() < 5 { return Err("Invalid table column add command arguments length".to_string()) }
                let column_args = &mut args[4..].iter().map(|s| s.as_str()).collect::<Vec<&str>>();
                column_args.remove(0);
                let column_type = args[4].as_str();
                let column = Self::parse_new_column(column_name, column_type, &column_args)?;
                table.columns.push(column)
            },

            _ => return Err(format!("Unknown column command: {}", command))
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
            "create" => schema.add_table(table_name)?,
            "delete" => schema.delete_table(table_name),
            "set" => Self::handle_table_set(schema, args, args_parts)?,
            "column" => Self::handle_table_column(schema, args, args_parts)?,
            "row" => Self::handle_table_row(schema, args, args_parts)?,

            _ => return Err(format!("Unknown table command: {}", args_parts[1]))
        }

        Ok(())
    }
}

impl Parser for SimpleParser {
    fn parse(input: &str) -> Result<Schema, String> {
        let mut schema = Schema::default(); 

        for line in input.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 0 { continue }
            if parts[0].starts_with("#") { continue }
            if parts.len() < 2 { return Err(format!("Invalid command: {}", line).to_string()) }

            let command = String::from(parts[0]).to_lowercase();
            let args_parts = &parts[1..];
            let args: Vec<String> = args_parts.iter().map(|s| s.to_string().to_lowercase()).collect();

            match command.as_str() {
                "table" => Self::handle_table(&mut schema, &args, args_parts)?,

                _ => return Err(format!("Unknown command: {}", command))
            }
        }

        Ok(schema)
    }

    fn parse_file(file: &str) -> Result<Schema, String> {
        log::info(format!("Parsing schema '{}'", file));

        match fs::read_to_string(file) {
            Ok(v) => Self::parse(&v),
            Err(e) => {
                log::error(format!("failed to read file {}\n{}", file, e));
                Err(e.to_string())
            }
        }
    }
}
