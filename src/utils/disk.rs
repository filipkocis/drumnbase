use std::fs;

use crate::utils::log;

pub fn copy_file(from: &str, to: &str) -> Result<(), String> {
    let result = fs::copy(from, to);

    if let Err(e) = result {
        let err_msg = format!("failed to copy file '{}' to '{}'\n{}", from, to, e);
        log::error(&err_msg);
        return Err(err_msg);
    }

    log::info(format!("copied file '{}' to '{}'", from, to)); 
    Ok(())
}

pub fn create_directory(path: &str) -> Result<(), String> {
    let result = fs::create_dir(path);
    
    if let Err(e) = result {
        let err_msg = format!("failed to create dir {}\n{}", path, e);
        log::error(&err_msg);
        return Err(err_msg);
    }

    log::info(format!("created dir {}", path)); 
    Ok(())
}

pub fn create_directory_all(path: &str) -> Result<(), String> {
    let result = fs::create_dir_all(path);

    if let Err(e) = result {
        let err_msg = format!("failed to create dir path {}\n{}", path, e);
        log::error(&err_msg);
        return Err(err_msg);
    }

    log::info(format!("created dir path {}", path)); 
    Ok(())
}

pub fn create_file(path: &str) -> Result<(), String> {
    let result = std::fs::OpenOptions::new().write(true).create_new(true).open(path);

    if let Err(e) = result {
        let err_msg = format!("failed to create file {}\n{}", path, e);
        log::error(&err_msg);
        return Err(err_msg);
    }

    log::info(format!("created file {}", path)); 
    Ok(())
}

pub fn get_entires(path: &str) {
    let result = std::fs::read_dir(path);


    match result {
        Ok(entries) => {
            // for entry in entries {
            //     if let Ok(entry) = entry {
            //         println!("{:?} {:?}", entry.path(), entry.file_name())
            //     }
            // }

            let files = entries
                .collect::<Result<Vec<_>, _>>()
                .unwrap()
                .into_iter()
                .filter(|entry| {
                    let path = entry.path();
                    let extension = path.extension();
                    let metadata = entry.metadata();

                    if extension.is_none() || metadata.is_err() { return false; }

                    if !metadata.unwrap().is_file() { return false; }
                    if extension.unwrap() != "quack" { return false; }

                    true
                })
                .map(|entry| entry.file_name());

            for file in files {
                println!("{}", file.to_str().unwrap())
            }
        },
        Err(e) => println!("failed to read dir {}\n{}", path, e),
    }
}

pub fn exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}






pub fn init_dnb_folder(name: &str) {
    let path = format!("{}", name);
    let result = std::fs::create_dir(path);
    if let Err(e) = result {
        println!("failed to init dnb folder {}\n{}", name, e)
    }
    println!("created dnb dir {}", name);
}

pub fn create_db(name: &str) {
    let path = format!("data/{}", name); 
    let result = std::fs::create_dir(path);

    match result {
        Ok(_) => println!("created dir {}", name),
        Err(e) => println!("failed to create dir {}\n{}", name, e),
    }
} 
