use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use crate::Error;
use crate::c_macro::{self, CMacro};

pub fn proc_header_file(file_path: &Path) -> Result<Vec<CMacro>, Error> {

    let input_file = File::open(file_path)?;
    let reader = BufReader::new(input_file);
    
    c_macro::convert_string_to_cmacro(get_hal_macros(reader)?)
}

fn get_hal_macros(reader: BufReader<File>) -> Result<Vec<String>, Error> {
    Ok(reader
        .lines()
        .filter_map(Result::ok)
        .fold(
            (Vec::new(), false, String::new()),
            |(mut macros, is_multiline, mut current_macro), line| {
                let trimmed = line.trim();
                
                // Check the start and end of multi-line macros
                let is_macro_start = trimmed.starts_with("#define __HAL_");
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
        .map(|macro_def| macro_def.replace("\\", ""))  // Remove continuation character
        .collect::<Vec<String>>())
}


