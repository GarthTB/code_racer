use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn get_unique_path(text_path: &PathBuf) -> Result<String, &'static str> {
    let dir = text_path.parent().ok_or("无法获取被测文本的父目录")?;
    let old_name = text_path.file_stem().ok_or("无法获取被测文本的文件名")?;
    let prefix = old_name.to_str().unwrap_or("被测文本");
    let mut new_name = format!("{prefix}_编码报告.txt");
    let mut i: usize = 2;
    while dir.join(&new_name).exists() {
        new_name = format!("{prefix}_编码报告_{i}.txt");
        i += 1;
    }
    Ok(new_name)
}

pub(crate) fn save_to_file(
    text_path: &PathBuf,
    report: &Vec<String>,
) -> Result<String, &'static str> {
    let unique_path = get_unique_path(text_path)?;
    let mut file = File::create(&unique_path).map_err(|_| "无法创建报告文件")?;
    for line in report {
        file.write_all(line.as_bytes())
            .map_err(|_| "无法写入报告文件")?;
        file.write_all(b"\n").map_err(|_| "无法写入报告文件")?;
    }
    Ok(unique_path)
}

pub(crate) fn print_to_console(report: Vec<String>) {
    for line in report {
        println!("{line}");
    }
}
