use crate::route_connector::RouteConnector;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

/// 返回按首字符分组的词库，子词库中的键为词组，值为(编码, 击键当量)
pub(crate) fn load_dict(
    path: &PathBuf,
    mut punct_items: Vec<(String, String, usize)>,
    connector: &RouteConnector,
) -> Result<(HashMap<char, HashMap<String, (String, f64)>>, usize), &'static str> {
    println!("读取词库文件...");
    let file = File::open(path).map_err(|_| "无法打开词库文件")?;
    let mut dict_items = read_dict_items(file)?;
    println!("读取完成。共{}个条目。", dict_items.len());
    println!("结合标点符号排序并生成翻页、选重信息...");
    sort_items(&mut dict_items);
    sort_items(&mut punct_items);
    let (dict, max_word_len) = convert_items(dict_items, punct_items, connector);
    if dict.is_empty() {
        return Err("词库为空");
    }
    println!("处理完成。共{}个最优条目。", dict.len());
    Ok((dict, max_word_len))
}

/// 将词库文件中的一行解析为(词组, 编码, 优先级)
pub(crate) fn parse_dict_line(items: &mut Vec<(String, String, usize)>, line: &str) {
    let line = line.split('#').next().expect("无法解析文件中的注释");
    let parts: Vec<&str> = line.split('\t').collect();
    if parts.len() == 2 || parts.len() == 3 {
        let punct = parts[0].to_string();
        let code = parts[1].to_string();
        let priority = parts.get(2).map_or(0, |p| p.parse().unwrap_or(0));
        items.push((punct, code, priority));
    }
}

/// 读取词库文件，返回所有条目，每个条目为(词组, 编码, 优先级)
fn read_dict_items(file: File) -> Result<Vec<(String, String, usize)>, &'static str> {
    let mut items = Vec::with_capacity(65536);
    for line in BufReader::new(file).lines() {
        let line = line.map_err(|_| "无法读取词库文件中的一行")?;
        parse_dict_line(&mut items, &line);
    }
    Ok(items)
}

/// 排序条目，顺序：优先级降序、码长升序、词组升序、编码升序
fn sort_items(items: &mut Vec<(String, String, usize)>) {
    let code_len = |item: &(String, String, usize)| item.1.len() as f64 / item.0.len() as f64;
    items.sort_by(|a, b| {
        b.2.cmp(&a.2)
            .then(code_len(a).partial_cmp(&code_len(b)).expect("无法比较码长"))
            .then(a.0.cmp(&b.0))
            .then(a.1.cmp(&b.1))
    });
}

/// 转换条目为按首字符分组的词库，子词库中的键为词组，值为(编码, 击键当量)
fn convert_items(
    dict_items: Vec<(String, String, usize)>, // 已经过排序，优先级无用
    punct_items: Vec<(String, String, usize)>, // 已经过排序，优先级无用
    connector: &RouteConnector,
) -> (HashMap<char, HashMap<String, (String, f64)>>, usize) {
    // 生成唯一编码的方法
    let length = dict_items.len() + punct_items.len();
    let mut used_codes = HashSet::with_capacity(length);
    let id_char = vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
    let mut get_code = |code: &str| {
        let mut code = code.to_string();
        let mut i = 2;
        while used_codes.contains(&code) {
            if i == 2 {
                code.push('2');
            } else if i < 10 {
                code.pop();
                code.push(id_char[i]);
            } else {
                code.pop();
                code.push('='); // 翻页符
            }
            i += 1;
        }
        used_codes.insert(code.clone()); // 码位被占用，但不代表会用到这个编码
        code
    };

    // 装填词库，并记录最长的词组长度的方法
    let mut max_word_len = 0;
    let mut dict: HashMap<char, HashMap<String, (String, f64)>> = HashMap::with_capacity(65536);
    let mut fill_dict = |word: String, code: String| {
        max_word_len = max_word_len.max(word.chars().count()); // 记录最长的词组长度
        let real_code = get_code(&code);
        let time = connector.get_time(&real_code);
        let key = word.chars().next().expect("装填词库时遇到词组为空");
        if let Some(sub_dict) = dict.get_mut(&key) {
            if let Some((_, old_time)) = sub_dict.get(&word) {
                // 只插入最优条目
                if time < *old_time {
                    sub_dict.insert(word, (real_code, time));
                }
            } else {
                sub_dict.insert(word, (real_code, time));
            }
        } else {
            let mut sub_dict = HashMap::new();
            sub_dict.insert(word, (real_code, time));
            dict.insert(key, sub_dict);
        }
    };

    // 进行装填
    for (word, code, _) in dict_items {
        fill_dict(word, code);
    }
    for (punct, code, _) in punct_items {
        fill_dict(punct, code);
    }

    (dict, max_word_len)
}
