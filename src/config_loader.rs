use std::collections::{HashMap, HashSet};
use std::env::current_exe;
use std::fs::{File, read_to_string};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

fn get_config_path(name: &str) -> Result<PathBuf, &'static str> {
    let exe_path = current_exe().map_err(|_| "无法获取可执行文件路径")?;
    let config_dir = exe_path.parent().ok_or("无法获取程序目录")?;
    Ok(config_dir.join("config").join(name))
}

pub(crate) fn load_layout() -> Result<Vec<String>, &'static str> {
    println!("加载键盘布局配置...");
    let layout_path = get_config_path("layout.txt")?;
    let content = read_to_string(&layout_path).map_err(|_| "无法读取键盘布局文件")?;
    let layout_lines: Vec<String> = content.lines().map(|line| line.to_string()).collect();
    println!("加载完成。应为14行，实际为{}行。", layout_lines.len());
    Ok(layout_lines)
}

pub(crate) fn load_punct_items() -> Result<HashSet<(String, String, usize)>, &'static str> {
    println!("加载标点符号配置...");
    let punct_path = get_config_path("punct_dict.txt")?;
    let punct_file = File::open(&punct_path).map_err(|_| "无法打开标点符号文件")?;
    let items = crate::dict_loader::read_rime_file(punct_file, 32)?;
    println!("加载完成。默认为30项，实际为{}项。", items.len());
    Ok(items)
}

pub(crate) fn load_time_map() -> Result<HashMap<(char, char), f64>, &'static str> {
    println!("加载击键当量配置...");
    let time_map_path = get_config_path("time_map.txt")?;
    let time_map_file = File::open(&time_map_path).map_err(|_| "无法打开击键当量文件")?;

    let mut time_map = HashMap::with_capacity(4096);
    for line in BufReader::new(time_map_file).lines() {
        let line = line.map_err(|_| "无法读取击键当量文件中的一行")?;
        let parts: Vec<&str> = line.split('\t').collect();
        let keys: Vec<char> = parts[0].chars().collect();
        if parts.len() != 2 || keys.len() != 2 {
            println!("击键当量文件中有格式错误的行：{}", line);
            continue;
        }

        match parts[1].parse() {
            Err(_) => println!("无法解析此行的击键当量：{}", line),
            Ok(time_cost) => match time_map.get(&(keys[0], keys[1])) {
                Some(_) => println!("击键当量文件中有重复的键：{}", parts[0]),
                None => {
                    time_map.insert((keys[0], keys[1]), time_cost);
                }
            },
        }
    }

    println!("加载完成。默认为2116行，实际为{}行。", time_map.len());
    Ok(time_map)
}
