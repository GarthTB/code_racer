use std::collections::HashMap;
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

pub(crate) fn load_time_cost() -> Result<HashMap<String, f64>, &'static str> {
    println!("加载击键当量配置...");
    let time_cost_path = get_config_path("time_cost.txt")?;
    let time_cost_file = File::open(&time_cost_path).map_err(|_| "无法打开击键当量文件")?;

    let mut time_cost_map = HashMap::with_capacity(4096);
    for line in BufReader::new(time_cost_file).lines() {
        let line = line.map_err(|_| "无法读取击键当量文件中的一行")?;
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() != 2 {
            println!("击键当量文件中有格式错误的行：{}", line)
        } else if let Ok(time_cost) = parts[1].parse() {
            if let Some(_) = time_cost_map.insert(parts[0].to_string(), time_cost) {
                println!("击键当量文件中有重复的键：{}", parts[0])
            }
        } else {
            println!("无法解析此行的击键当量：{}", line)
        };
    }

    println!("加载完成。默认为2116行，实际为{}行。", time_cost_map.len());
    Ok(time_cost_map)
}

pub(crate) fn load_punct_items() -> Result<Vec<(String, String, usize)>, &'static str> {
    println!("加载标点符号配置...");
    let punct_path = get_config_path("punct_dict.txt")?;
    let punct_file = File::open(&punct_path).map_err(|_| "无法打开标点符号文件")?;

    let mut items = Vec::with_capacity(32);
    for line in BufReader::new(punct_file).lines() {
        let line = line.map_err(|_| "无法读取标点符号文件中的一行")?;
        crate::dict_loader::parse_dict_line(&mut items, &line);
    }

    println!("加载完成。默认为24行，实际为{}行。", items.len());
    Ok(items)
}
