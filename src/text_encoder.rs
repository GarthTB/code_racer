use crate::route_buffer::RouteBuffer;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::PathBuf;

pub(crate) fn encode(
    text_path: &PathBuf,
    dict: HashMap<char, Vec<(Vec<char>, Vec<char>, f64)>>,
    buffer: &mut RouteBuffer,
) -> Result<(Vec<char>, f64), &'static str> {
    println!("计算编码...");
    let text_string = read_to_string(text_path).map_err(|_| "无法读取待编码文本文件")?;
    let text_chars: Vec<char> = text_string.chars().collect();

    println!("共需计算{}字。计算编码...", text_chars.len());
    for i in 0..text_chars.len() {
        if i % 3000 == 0 {
            let count = buffer.unknown_keys_count();
            print!("\r已计算至第{i}字。遇到{}个找不到当量的按键组合。", count);
        }
        if let Some(sub_dict) = dict.get(&text_chars[i]) {
            for (word, code, time) in sub_dict {
                if text_chars[i..].starts_with(word) {
                    buffer.connect_code(word.len(), code, *time)
                }
            }
        }
        if !buffer.is_connected() {
            buffer.connect_code(1, &text_chars[i..i + 1], 0.0)
        }
        buffer.next();
    }
    let (route, time) = buffer.get_global_best_route()?;
    println!("\n计算完成。");

    Ok((route, time))
}
