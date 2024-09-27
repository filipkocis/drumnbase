use std::collections::HashMap;

/// Parses cli arguments into a hashmap of key value pairs
/// Format value is a tuple of (long, short, is_flag, is_required, default)
pub fn parsed_env_args(format: Vec<(&str, char, bool, bool, Option<&str>)>) -> Result<HashMap<String, String>, String> {
    let mut map = HashMap::new();
    let args: Vec<_> = std::env::args().collect();

    let mut i = 1;
    while i < args.len() {
        let arg_1 = args[i].clone();

        // Check if argument is a key
        let name: String;
        if arg_1.starts_with("--") {
            name = arg_1[2..].to_string();
        } else if arg_1.starts_with('-') {
            name = arg_1[1..].to_string();
        } else {
            return Err(format!("Invalid argument key: {}", arg_1))
        }

        if name.is_empty() {
            return Err(format!("Invalid argument key: {}", arg_1))
        }

        // Find the format entry for the argument key
        let format_entry = format.iter().find(|(l, s, ..)| { 
            if arg_1.starts_with("--") {
                l == &name
            } else {
                s == &name.chars().next().unwrap()
            }
        }).ok_or(format!("Unknown argument key: {}", arg_1))?;

        let name = format_entry.0.to_string();
        let is_flag = format_entry.2;

        // Early exit if we are at the end
        if i + 1 >= args.len() {
            match is_flag {
                true => {
                    map.insert(name, "".to_string()); 
                    break;
                },
                false => return Err(format!("Argument {} requires a value", arg_1))
            }
        }

        let arg_2 = args[i + 1].clone();
        let is_arg_2_value = !arg_2.starts_with('-');

        match (is_arg_2_value, is_flag) {
            // Error if arg requires a value but none is provided, 
            // or if it doesn't but one is provided
            (false, false) => return Err(format!("Argument {} requires a value", arg_1)),
            (true, true) => return Err(format!("Argument {} does not require a value", arg_1)),
            // Insert key value pair, or just the key if it's a flag
            (false, true) => {
                map.insert(name, "".to_string());
                i += 1;
            },
            (true, false) => {
                map.insert(name, arg_2);
                i += 2;
            }
        }
    }

    for (name, _, _, is_required, default) in format {
        if map.contains_key(name) {
            continue;
        }

        match (is_required, default) {
            (false, None) => continue,

            // Error if a required key is missing
            (true, None) => return Err(format!("Required argument {} is missing", name)),

            // Insert default values for missing keys
            (_, Some(default)) => map.insert(name.to_string(), default.to_string()),
        };
    }

    Ok(map)
}
