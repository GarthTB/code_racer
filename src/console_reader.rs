use std::path::PathBuf;

fn read() -> String {
    let mut input = String::new();
    loop {
        if let Ok(_) = std::io::stdin().read_line(&mut input) {
            return input.trim().to_string();
        }
        println!("无法读取输入。请重新输入。");
    }
}

fn get_dict_path() -> PathBuf {
    println!("请输入词库文件路径：");
    loop {
        let path = PathBuf::from(read());
        if path.exists() {
            return path;
        }
        println!("文件不存在。请重新输入。");
    }
}

fn get_method_code() -> usize {
    println!("请输入连接方法代号：0-空格或符号，1-无间隔，2-键道顶功");
    loop {
        if let Ok(code) = read().parse() {
            if code < 3 {
                return code;
            }
        }
        println!("无效代号。请重新输入。");
    }
}

fn get_text_path() -> PathBuf {
    println!("请输入待编码文本文件路径：");
    loop {
        let path = PathBuf::from(read());
        if path.exists() {
            return path;
        }
        println!("文件不存在。请重新输入。");
    }
}
