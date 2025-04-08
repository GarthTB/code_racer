use crate::dict_loader::load_dict;
use crate::route_connector::RouteConnector;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub(crate) fn read_line() -> String {
    let mut input = String::new();
    loop {
        match std::io::stdin().read_line(&mut input) {
            Ok(_) => return input.trim().to_string(),
            Err(message) => println!("无法读取输入。错误信息：{message}。请重新输入。"),
        }
    }
}

pub(crate) fn get_connector(time_map: HashMap<(char, char), f64>) -> RouteConnector {
    println!("请输入连接方法代号：");
    println!("0: 空格或符号; 1: 无间隔; 2: 键道顶功");
    loop {
        if let Ok(code) = read_line().parse() {
            if code < 3 {
                return RouteConnector::new(time_map, code);
            }
        }
        println!("无效代号。请重新输入。")
    }
}

pub(crate) fn get_dict(
    punct_items: HashSet<(String, String, usize)>,
    connector: RouteConnector,
) -> (HashMap<char, Vec<(Vec<char>, Vec<char>, f64)>>, usize) {
    println!("请输入词库文件路径：");
    loop {
        let path = PathBuf::from(read_line());
        match path.exists() {
            true => match load_dict(&path, punct_items.clone(), connector.clone()) {
                Ok((dict, max_word_len)) => return (dict, max_word_len),
                Err(message) => println!("无法加载词库。错误信息：{message}。请重新输入。"),
            },
            false => println!("文件不存在。请重新输入。"),
        }
    }
}

pub(crate) fn get_text_path() -> PathBuf {
    println!("请输入待编码文本文件路径：");
    loop {
        let path = PathBuf::from(read_line());
        match path.exists() {
            true => return path,
            false => println!("文件不存在。请重新输入。"),
        }
    }
}

pub(crate) fn need_to_report_unknown_keys(count: usize) -> bool {
    println!("是否需要输出这{count}个找不到当量的按键组合？");
    println!("随便输入一个数字以确认；输入其他则取消...");
    read_line().parse::<f64>().is_ok()
}
