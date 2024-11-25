use std::fs::{self, create_dir_all, File};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IoError")]
    IoError(#[from] io::Error),
}

pub fn proc_header_file(file_path: &str) -> Result<String, Error> {
    let path = Path::new(file_path);
    let file_name = path.file_name().unwrap().to_str().unwrap();

    let input_file = File::open(path)?;
    let reader = BufReader::new(input_file);
    
    get_macros(reader).map(|macros| macros.join("\n"))

    // let out_macros = get_macros(reader)?;

    // create_dir_all("output/temp")?;
    
    // let output_path = format!("output/temp/{}_macros.h", file_name);
    // let mut output_file = File::create(&output_path)?;
    // writeln!(output_file, "/* Extracted macros from {} */\n", file_path)?;

    // output_file.write_all(out_macros.as_bytes())?;


    // Ok(())
}

fn get_macros(reader: BufReader<File>) -> Result<Vec<String>, Error> {
    Ok(reader
        .lines()
        .filter_map(Result::ok)
        .fold(
            (Vec::new(), false, String::new()),
            |(mut macros, is_multiline, mut current_macro), line| {
                let trimmed = line.trim();
                
                // Check the start and end of multi-line macros
                let is_macro_start = trimmed.starts_with("#define");
                let is_line_continued = trimmed.ends_with("\\");
                
                match (is_macro_start, is_multiline, is_line_continued) {
                    (true, false, true) => {
                        // Start of a new multi-line macro
                        (macros, true, trimmed.to_string())
                    }
                    (true, false, false) => {
                        // Single-line macro
                        macros.push(trimmed.to_string());
                        (macros, false, String::new())
                    }
                    (false, true, true) => {
                        // Intermediate line of a multi-line macro
                        current_macro.push_str(trimmed);
                        (macros, true, current_macro)
                    }
                    (false, true, false) => {
                        // Last line of a multi-line macro
                        current_macro.push_str(trimmed);
                        macros.push(current_macro);
                        (macros, false, String::new())
                    }
                    _ => (macros, is_multiline, current_macro)
                }
            },
        )
        .0
        .into_iter()
        .map(|macro_def| macro_def.replace("\\/", ""))  // Remove continuation character
        .collect::<Vec<String>>())
}
