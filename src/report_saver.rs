use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn save_to_file(
    text_path: &PathBuf,
    name: &str,
    content: &Vec<String>,
) -> Result<String, &'static str> {
    // 生成绝对路径
    let dir = text_path.parent().ok_or("无法获取被测文本的父目录")?;
    let old_name = text_path.file_stem().ok_or("无法获取被测文本的文件名")?;
    let prefix = old_name.to_str().ok_or("无法获取被测文本的文件名")?;
    let mut new_name = format!("{prefix}_{name}.txt");
    let mut i: usize = 2;
    while dir.join(&new_name).exists() {
        new_name = format!("{prefix}_{name}_{i}.txt");
        i += 1;
    }
    let unique_path = dir
        .join(&new_name)
        .to_str()
        .ok_or("无法生成唯一的报告文件路径")?
        .to_string();

    // 写入文件
    let mut file = File::create(&unique_path).map_err(|_| "无法创建报告文件")?;
    for line in content {
        file.write_all(line.as_bytes())
            .map_err(|_| "无法写入报告文件")?;
        file.write_all(b"\n").map_err(|_| "无法写入报告文件")?;
    }

    Ok(unique_path)
}

pub(crate) fn save(text_path: &PathBuf, name: &str, content: Vec<String>) {
    println!("保存{name}...");
    match save_to_file(text_path, name, &content) {
        Ok(unique_path) => {
            println!("{name}已保存至：{unique_path}");
        }
        Err(message) => {
            println!("无法将{name}保存至文件。错误信息：{message}");
            println!("将直接输出到控制台...");
            for line in content {
                println!("{line}");
            }
            println!("{name}输出完毕。");
        }
    }
}
