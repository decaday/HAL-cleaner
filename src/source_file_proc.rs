use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::Path;
use std::process::Output;

use tree_sitter::Parser;

use crate::Error;
use crate::c_macro::{self, CMacro};

pub fn proc_source_file(input_path: &Path, output_path: &Path, c_macros: Option<Vec<CMacro>>) -> Result<(), Error> {

    fs::create_dir_all("output/temp")?;
    let temp_path = if let Some(c_macros) = c_macros {
        let temp_puth = Path::new("output/temp/preproc_out.c");
        c_macro::process_c_macros(&c_macros, input_path, temp_puth).unwrap();
        temp_puth
    }
    else {
        input_path
    };
    
    // parse(temp_path, output_path)?;

    Ok(())
}

fn parse(input_path: &Path, output_path: &Path) -> Result<(), Error> {
    let mut source_code = String::new();
    File::open(input_path)?.read_to_string(&mut source_code)?;

    let mut output_file = File::create(output_path)?;

    let mut parser = Parser::new();
    
    let language =tree_sitter_c::LANGUAGE.into();
    parser.set_language(&language).unwrap();

    let tree = parser.parse(&source_code, None).unwrap();
    let root_node = tree.root_node();

    // 记录上次的字节位置，初始化为0
    let mut last_byte_pos = 0;

    for node in root_node.children(&mut root_node.walk()) {
        if node.kind() == "function_item" {
            // let function_text = parse_function(node, source_code);
            
            let current_start = node.start_byte();
            output_file.write_all(&source_code.as_bytes()[last_byte_pos..current_start])?;
            
            // output_file.write_all(function_text.as_bytes())?;
            
            last_byte_pos = node.end_byte();
        }
    }

    Ok(())
}