use std::fs;

use crate::{
    parser::Schema, 
    file::data::LoadMode, 
    basics::column::{Column, ColumnType, NumericType, TextType}
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
            "variable" => ColumnType::Text(TextType::Variable),
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

            _ => return Err(format!("unknown column type: {}", column_type_str).to_string())
        };

        let mut column = Column::new(column_name, column_type);

        for &arg in args {
            let parts: Vec<&str> = arg.split("=").collect();

            match parts[0] {
                "length" => {
                    if parts.len() !=2 { return Err("Invalid column length argument".to_string()) }
                    column.length = parts[1].parse().unwrap();
                },
                "default" => {
                    if parts.len() !=2 { return Err("Invalid column length argument".to_string()) }
                    let default_value = parts[1];

                    match column.data_type {
                        ColumnType::Numeric(_) => {
                            if default_value.parse::<f64>().is_err() {
                                return Err(format!("Invalid default value for numeric column: {}", default_value));
                            };
                        },
                        _ => { }
                    };
                    column.default = default_value.to_string();
                },
                "not_null" => column.not_null = true,
                "unique" => column.unique = true,
                "read_only" => column.read_only = true,

                _ => return Err(format!("Unknown new column property {}", arg).to_string())
            }
        }

        Ok(column)
    }

    /// table <table_name> column <column_name> <column_command> <column_type> [column_args...]
    fn handle_table_column(schema: &mut Schema, args: &Vec<String>, args_parts: &[&str]) -> Result<(), String> {
        if args.len() < 4 { return Err("Invalid table column command arguments".to_string()) }

        let table_name = args_parts[0];
        let column_name = args_parts[2];
        let command = args[3].as_str();

        let table = schema.get_table(table_name)
            .ok_or(format!("Table not found: {}", table_name))?;

        match command {
            "delete" => table.columns.retain(|c| c.name != column_name),
            "add" => {
                if args.len() < 5 { return Err("Invalid table column add command arguments".to_string()) }
                let column_args = &mut args[4..].iter().map(|s| s.as_str()).collect::<Vec<&str>>();
                column_args.remove(0);
                let column_type = args[4].as_str();
                let column = Self::parse_new_column(column_name, column_type, &column_args)?;
                table.columns.push(column)
            }

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

        fs::create_dir_all(&format!("{}/tables", schema.root_dir)).unwrap();

        Ok(())
    }
}

impl Parser for SimpleParser {
    fn parse(input: &str) -> Result<Schema, String> {
        let mut schema = Schema::default(); 

        for line in input.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 0 { continue }
            if parts.len() < 2 { return Err(format!("Invalid command: {}", line).to_string()) }

            let command = String::from(parts[0]).to_lowercase();
            let args_parts = &parts[1..];
            let args: Vec<String> = args_parts.iter().map(|s| s.to_string().to_lowercase()).collect();



            match command.as_str() {
                "root_dir" => Self::handle_root_dir(&mut schema, &args, args_parts)?,
                "table" => Self::handle_table(&mut schema, &args, args_parts)?,

                _ => return Err(format!("Unknown command: {}", command))
            }

        }
        
        Ok(schema)
    }

    fn parse_file(file: &str) -> Result<Schema, String> {
        if let Ok(str) = fs::read_to_string(file) {
            return Self::parse(&str);
        } else {
            return Err("Could not read file".to_string())
        }
    }
}
