use crate::dict_loader;
use crate::route_connector::RouteConnector;
use std::collections::HashMap;
use std::path::PathBuf;

pub(crate) fn read() -> String {
    let mut input = String::new();
    loop {
        match std::io::stdin().read_line(&mut input) {
            Ok(_) => return input.trim().to_string(),
            Err(_) => println!("无法读取输入。请重新输入。"),
        }
    }
}

pub(crate) fn get_connector(time_map: HashMap<String, f64>) -> RouteConnector {
    println!("请输入连接方法代号：");
    println!("0: 空格或符号; 1: 无间隔; 2: 键道顶功");
    loop {
        if let Ok(code) = read().parse() {
            if code < 3 {
                return RouteConnector::new(time_map, code);
            }
        }
        println!("无效代号。请重新输入。")
    }
}

pub(crate) fn get_dict(
    punct_items: Vec<(String, String, usize)>,
    connector: &RouteConnector,
) -> (HashMap<char, HashMap<String, (String, f64)>>, usize) {
    println!("请输入词库文件路径：");
    loop {
        let path = PathBuf::from(read());
        match path.exists() {
            true => match dict_loader::load_dict(&path, punct_items.clone(), connector) {
                Ok((dict, max_word_len)) => return (dict, max_word_len),
                Err(e) => println!("{e}。请重新输入。"),
            },
            false => println!("文件不存在。请重新输入。"),
        }
    }
}

pub(crate) fn get_text() -> Vec<char> {
    println!("请输入待编码文本文件路径：");
    loop {
        let path = PathBuf::from(read());
        match path.exists() {
            true => match std::fs::read_to_string(path) {
                Ok(text) => match text.is_empty() {
                    false => return text.chars().collect(),
                    true => println!("文件为空。请重新输入。"),
                },
                Err(_) => println!("无法读取文件。请重新输入。"),
            },
            false => println!("文件不存在。请重新输入。"),
        }
    }
}
