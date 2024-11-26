use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use regex::Regex;

use crate::Error;

#[derive(Clone, Debug)]
pub struct CMacro {
    pub name: String,
    pub params: Option<Vec<String>>,
    pub content: String,
}

pub fn process_c_macros(macros: &[CMacro], input_file: &Path, output_file: &Path) -> Result<(), Error> {
    // 创建输出目录
    fs::create_dir_all("output/temp")?;
    
    // 打开输入和输出文件
    let file = File::open(input_file)?;
    let reader = BufReader::new(file);
    let mut writer = File::create(output_file)?;

    // 按行读取并缓冲
    let mut buffer = String::new();
    
    for line in reader.lines() {
        let line = line?.replace("#ifdef ", "//HC #ifdef ")
            .replace("#ifndef ", "//HC #ifndef ")
            .replace("#if ", "//HC #if ")
            .replace("#else ", "//HC #else ")
            .replace("#endif ", "//HC #endif ");

        buffer.push_str(&line);
        buffer.push('\n'); // 保留换行，确保宏可以跨多行解析

        // 处理到完整的语句
        while let Some(pos) = buffer.find(';') {
            let (statement, remainder) = buffer.split_at(pos + 1); // 包括分号
            let mut processed_statement = statement.to_string();
            buffer = remainder.trim_start().to_string(); // 去掉前导空格

            // 遍历所有宏定义并尝试替换
            for macro_def in macros {
                processed_statement = expand_c_macro(macro_def, &processed_statement);
            }

            // 写入处理后的语句
            writeln!(writer, "{}", processed_statement)?;
        }
    }

    // 处理缓冲区中剩余的内容（如果存在未闭合的语句）
    if !buffer.trim().is_empty() {
        let mut processed_statement = buffer.to_string();
        for macro_def in macros {
            processed_statement = expand_c_macro(macro_def, &processed_statement);
        }
        writeln!(writer, "{}", processed_statement)?;
    }

    Ok(())
}

fn expand_c_macro(macro_def: &CMacro, input_statement: &str) -> String {
    if !input_statement.contains(&macro_def.name) {
        return input_statement.to_string();
    }

    // 如果宏没有参数，直接替换
    if macro_def.params.is_none() {
        return input_statement.replace(&macro_def.name, &macro_def.content);
    }

    // 有参数的宏处理
    let params = macro_def.params.as_ref().unwrap();
    let mut expanded_statement = input_statement.to_string();

    // 尝试提取括号中的参数
    if let Some(start) = expanded_statement.find(&macro_def.name) {
        if let Some(open_paren) = expanded_statement[start..].find('(') {
            let end_paren = expanded_statement[start..].find(')').unwrap_or(expanded_statement.len());
            let param_str = &expanded_statement[start + open_paren + 1 .. start + end_paren];
            
            // 分割参数
            let actual_params: Vec<&str> = param_str.split(',')
                .map(|p| p.trim())
                .collect();

            // 检查参数个数是否匹配
            if actual_params.len() == params.len() {
                let mut replacement = macro_def.content.clone();
                for (i, param) in params.iter().enumerate() {
                    replacement = replacement.replace(param, actual_params[i]);
                }
                
                // 替换整个宏调用
                expanded_statement = expanded_statement.replace(
                    &expanded_statement[start..start + end_paren + 1], 
                    &replacement
                );
            }
        }
    }

    expanded_statement
}

pub fn convert_string_to_cmacro(macros: Vec<String>) -> Result<Vec<CMacro>, Error> {
    Ok(macros.into_iter()
        .map(|macro_str| {
            // 使用正则表达式解析宏定义
            let macro_regex = Regex::new(r"#define\s*(\w+)\s*\((.*?)\)\s*([\s\S]*)").unwrap();
            
            let captures = macro_regex.captures(&macro_str).expect("Invalid macro format");
            
            // 提取名称
            let name = captures.get(1).map_or("", |m| m.as_str()).to_string();
            
            // 解析参数
            let params: Vec<String> = captures.get(2)
                .map_or(Vec::new(), |m| 
                    m.as_str()
                     .split(',')
                     .map(|p| p.trim().to_string())
                     .filter(|p| !p.is_empty())
                     .collect()
                );
            
            // 处理宏内容
            let mut content = captures.get(3)
                .map_or("", |m| m.as_str())
                .trim()
                .to_string();
            
            // 替换多个空格为两个空格
            content = content.split_whitespace().collect::<Vec<&str>>().join(" ");
            
            // 删除开头的do{和末尾的} while(0)
            if content.starts_with("do{") && content.ends_with("} while(0)") {
                content = content[3..content.len()-10].trim().to_string();
            }
            
            CMacro {
                name,
                params: Some(params),
                content,
            }
        })
        .collect())
}


// 测试模块
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_conversion() {
        let input = vec![
            "#define __HAL_ADC_CLEAR_CALIBFAIL_FLAG(__HANDLE__) (((__HANDLE__)->Instance->CCSR) |= ADC_CCSR_CALFAIL)".to_string(),
            "#define __HAL_ADC_RESET_HANDLE_STATE(__HANDLE__)                               do{                                                                          (__HANDLE__)->State = HAL_ADC_STATE_RESET;
               (__HANDLE__)->MspInitCallback = NULL;                                     (__HANDLE__)->MspDeInitCallback = NULL;                                   } while(0)".to_string(),
            "#define MAX(a, b) ((a) > (b) ? (a) : (b))".to_string()
        ];

        let result = convert_string_to_cmacro(input).unwrap();

        assert_eq!(result.len(), 3);
        
        assert_eq!(result[0].name, "__HAL_ADC_CLEAR_CALIBFAIL_FLAG");
        assert_eq!(result[0].params, Some(vec!["__HANDLE__".to_string()]));
        assert_eq!(result[0].content, "(((__HANDLE__)->Instance->CCSR) |= ADC_CCSR_CALFAIL)");
        
        assert_eq!(result[1].name, "__HAL_ADC_RESET_HANDLE_STATE");
        assert_eq!(result[1].params, Some(vec!["__HANDLE__".to_string()]));
        assert_eq!(result[1].content, "(__HANDLE__)->State = HAL_ADC_STATE_RESET; (__HANDLE__)->MspInitCallback = NULL; (__HANDLE__)->MspDeInitCallback = NULL;");
        
        assert_eq!(result[2].name, "MAX");
        assert_eq!(result[2].params, Some(vec!["a".to_string(), "b".to_string()]));
        assert_eq!(result[2].content, "((a) > (b) ? (a) : (b))");
    }

    #[test]
    fn test_multiple_spaces_replacement() {
        let input = vec![
            "#define EXAMPLE_MACRO(param)   some    long       macro    content".to_string()
        ];

        let result = convert_string_to_cmacro(input).unwrap();

        assert_eq!(result[0].name, "EXAMPLE_MACRO");
        assert_eq!(result[0].params, Some(vec!["param".to_string()]));
        assert_eq!(result[0].content, "some long macro content");
    }

    #[test]
    fn test_macro_replacement() {
        // 创建测试输入文件
        fs::create_dir_all("output/temp").unwrap();
        let test_input_path = Path::new("output/temp/temp_macros.c");
        let test_output_path = Path::new("output/temp/temp_macros_output.c");
        let mut test_input = File::create(&test_input_path).unwrap();
        
        // 写入测试输入
        writeln!(test_input, "int main() {{").unwrap();
        writeln!(test_input, "    int x = 5, y = 10;").unwrap();
        writeln!(test_input, "    int max_val = MAX(x, y);").unwrap();
        writeln!(test_input, "    float circle_area = PI * x * x;").unwrap();
        writeln!(test_input, "    return 0;").unwrap();
        writeln!(test_input, "}}").unwrap();

        // 定义宏
        let macros = vec![
            CMacro {
                name: "MAX".to_string(),
                params: Some(vec!["a".to_string(), "b".to_string()]),
                content: "((a) > (b) ? (a) : (b))".to_string(),
            },
            CMacro {
                name: "PI".to_string(),
                params: None,
                content: "3.14159".to_string(),
            }
        ];

        // 处理宏替换
        process_c_macros(&macros, test_input_path, test_output_path).unwrap();

        // 读取输出文件并验证
        let output_content = fs::read_to_string(test_output_path).unwrap();
        
        assert!(output_content.contains("int max_val = ((x) > (y) ? (x) : (y));"));
        assert!(output_content.contains("float circle_area = 3.14159 * x * x;"));
    }
}
