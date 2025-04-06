use crate::route_buffer::RouteBuffer;
use std::collections::HashMap;

pub(crate) fn encode(
    text: &Vec<char>,
    dict: HashMap<char, Vec<(String, String, f64)>>,
    mut buffer: RouteBuffer,
) -> Result<(String, f64), &'static str> {
    println!("共需计算{}字。计算编码...", text.len());
    for i in 0..text.len() {
        if let Some(sub_dict) = dict.get(&text[i]) {
            for (word, code, time) in sub_dict {
                let chars: Vec<char> = word.chars().collect();
                if text[i..].starts_with(&chars) {
                    buffer.connect_code(chars.len(), code, *time)
                }
            }
        }
        if !buffer.is_connected() {
            buffer.connect_code(1, text[i].to_string().as_str(), 0.0)
        }
        buffer.next();
        print!("\r已计算至第{}字。", i + 1);
    }
    println!("计算完成。");
    buffer.get_global_best_route()
}
