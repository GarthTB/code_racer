use crate::route_connector::RouteConnector;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

/// 将Rime格式词库文件中的每一行(词组, 编码, 优先级)解析并返回一个HashSet
pub(crate) fn read_rime_file(
    file: File,
    capacity: usize,
) -> Result<HashSet<(String, String, usize)>, &'static str> {
    let mut items = HashSet::with_capacity(capacity);
    for line in BufReader::new(file).lines() {
        let line = line.map_err(|_| "无法读取文件中的一行")?;
        let item = line.split('#').next().expect("无法解析文件中的注释");
        let parts: Vec<&str> = item.split('\t').collect();
        if parts.len() == 2 {
            items.insert((parts[0].to_string(), parts[1].to_string(), 0));
        } else if parts.len() == 3 {
            items.insert((
                parts[0].to_string(),
                parts[1].to_string(),
                parts[2].parse().unwrap_or(0),
            ));
        }
    }
    Ok(items)
}

/// 返回按首字符分组的词库，以及其中词组的最大长度。子词库中的元素为(词组, 编码, 击键当量)
pub(crate) fn load_dict(
    path: &PathBuf,
    punct_items: HashSet<(String, String, usize)>,
    connector: RouteConnector, // 克隆一个，和用于编码的连接器区分开，不要借用
) -> Result<(HashMap<char, Vec<(Vec<char>, Vec<char>, f64)>>, usize), &'static str> {
    println!("读取词库文件...");
    let file = File::open(path).map_err(|_| "无法打开词库文件")?;
    let dict_items = read_rime_file(file, 65536)?;
    println!("读取完成。共{}个条目。", dict_items.len());
    println!("结合标点符号排序并生成翻页、选重信息...");
    let sorted_dict_items = sort_items(&dict_items);
    let sorted_punct_items = sort_items(&punct_items);
    let (dict, max_word_len) = convert_items(sorted_dict_items, sorted_punct_items, connector);
    if dict.is_empty() {
        return Err("词库为空");
    }
    println!("处理完成。首字共覆盖{}个字符。", dict.len());
    Ok((dict, max_word_len))
}

/// 排序条目。顺序：优先级降序、码长升序、词升序、码升序
fn sort_items(items: &HashSet<(String, String, usize)>) -> Vec<(String, String, usize)> {
    let code_len = |w: &str, c: &str| c.len() as f64 / w.len() as f64;
    let mut sorted: Vec<_> = items.into_iter().cloned().collect();
    sorted.sort_by(|(w1, c1, p1), (w2, c2, p2)| {
        p2.cmp(p1)
            .then(
                code_len(w1, c1)
                    .partial_cmp(&code_len(w2, c2))
                    .expect("无法比较码长"),
            )
            .then(w1.cmp(w2))
            .then(c1.cmp(c2))
    });
    sorted
}

/// 转换条目为按首字符分组的词库，并返回其中词组的最大长度。子词库中的元素为(词组, 编码, 击键当量)
fn convert_items(
    dict_items: Vec<(String, String, usize)>, // 已经过排序，优先级无用
    punct_items: Vec<(String, String, usize)>, // 已经过排序，优先级无用
    mut connector: RouteConnector,
) -> (HashMap<char, Vec<(Vec<char>, Vec<char>, f64)>>, usize) {
    // 生成唯一编码的方法
    let count = dict_items.len() + punct_items.len();
    let mut used_codes = HashSet::with_capacity(count);
    let mut get_unique_code = |code: &str| {
        let mut unique_code = code.to_string();
        let mut i: u8 = 2;
        while used_codes.contains(&unique_code) {
            if i == 2 {
                unique_code.push('2');
            } else if i < 10 {
                unique_code.pop();
                unique_code.push(('0' as u8 + i) as char);
            } else {
                unique_code.pop();
                unique_code.push('='); // 等号翻页
                i = 1;
            }
            i += 1;
        }
        used_codes.insert(unique_code.clone()); // 码位被占用，但不代表会用到这个编码
        unique_code
    };

    // 装填词库的方法，编码拆成数组
    let mut mid_dict: HashMap<String, (Vec<char>, f64)> = HashMap::with_capacity(65536);
    let mut add_item = |word: String, code: String| {
        let new_code = get_unique_code(&code).chars().collect();
        let new_time = connector.get_time(&new_code);
        if let Some((old_code, old_time)) = mid_dict.get(&word) {
            if new_time < *old_time || new_code.len() < old_code.len() {
                mid_dict.insert(word, (new_code, new_time));
            }
        } else {
            mid_dict.insert(word, (new_code, new_time));
        }
    };

    // 进行装填
    for (word, code, _) in dict_items {
        add_item(word, code);
    }
    for (punct, code, _) in punct_items {
        add_item(punct, code);
    }
    println!("整理后共{}个最优词组。", mid_dict.len());
    if connector.unknown_keys_count() == 0 {
        println!("编码中没有遇到找不到当量的按键组合。");
    } else {
        println!(
            "编码中遇到{}个找不到当量的按键组合。",
            connector.unknown_keys_count()
        );
    }

    // 词组拆成数组，按首字分组，并记录最长词组长度
    let mut max_word_len = 0;
    let mut master_dict: HashMap<char, Vec<_>> = HashMap::with_capacity(16384);
    for (word, (code, time)) in mid_dict {
        let word_chars: Vec<char> = word.chars().collect();
        max_word_len = max_word_len.max(word_chars.len());
        if let Some(sub_dict) = master_dict.get_mut(&word_chars[0]) {
            sub_dict.push((word_chars, code, time));
        } else {
            master_dict.insert(word_chars[0], vec![(word_chars, code, time)]);
        }
    }
    println!("最大词组长度为{}个字。", max_word_len);

    (master_dict, max_word_len)
}
