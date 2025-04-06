use std::collections::HashMap;

pub(crate) struct RouteConnector {
    /// 按键之间的用时当量，键为两个按键，值为用时当量
    time_map: HashMap<String, f64>,
    /// 连接方法代号：0-空格或符号，1-无间隔，2-键道顶功
    method_code: usize,
}

impl RouteConnector {
    pub(crate) fn new(time_map: HashMap<String, f64>, method_code: usize) -> RouteConnector {
        RouteConnector {
            time_map,
            method_code,
        }
    }

    pub(crate) fn get_time(&self, text: &str) -> f64 {
        let mut sum = 0.0;
        let mut chars = text.chars().peekable();
        let mut key = String::with_capacity(2);

        while let Some(c1) = chars.next() {
            if let Some(&c2) = chars.peek() {
                key.clear();
                key.push(c1);
                key.push(c2);

                match self.time_map.get(&key) {
                    Some(value) => sum += value,
                    None => {
                        println!("找不到{key}对应的当量。已默认为1.4。");
                        sum += 1.4;
                    }
                }
            }
        }

        sum
    }

    pub(crate) fn connect(&self, s1: &str, t1: f64, s2: &str, t2: f64) -> (String, f64) {
        let a = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let n = "0123456789";

        let s1_last = if s1.is_empty() {
            '\0'
        } else {
            s1.chars().next_back().expect("无法获取前部末字符。")
        };
        let (s2_first, s2_last) = if s2.is_empty() {
            ('\0', '\0')
        } else {
            (
                s2.chars().next().expect("无法获取后部首字符。"),
                s2.chars().next_back().expect("无法获取后部末字符。"),
            )
        };

        let directly = || {
            let s: String = s1.chars().chain(s2.chars()).collect();
            let mut m = String::with_capacity(2);
            m.push(s1_last);
            m.push(s2_first);
            let t = t1 + t2 + self.get_time(&m);
            (s, t)
        };

        let directly_with_space = || {
            let mut s = s1.to_string();
            s.push(' ');
            s.push_str(s2);
            let mut m = String::with_capacity(3);
            m.push(s1_last);
            m.push(' ');
            m.push(s2_first);
            let t = t1 + t2 + self.get_time(&m);
            (s, t)
        };

        match self.method_code {
            0 => {
                if t1 == 0.0 {
                    (s2.to_string(), t2)
                } else if a.contains(s1_last) && (a.contains(s2_first) || n.contains(s2_first)) {
                    directly_with_space() // 字母 + (字母或数字)，则加空格
                } else {
                    directly()
                }
            }
            1 => {
                if t1 == 0.0 {
                    (s2.to_string(), t2)
                } else {
                    directly()
                }
            }
            2 => {
                let x = "aiouv"; // 形码码元
                let y = "bcdefghjklmnpqrstwxyz"; // 音码码元

                let mut s2_chars: Vec<char> = s2.chars().collect();
                let mut mod_t2 = t2;

                // 新码以音码结尾，且不足4码：后补空格
                if y.contains(s2_last) && s2_chars.len() < 4 {
                    s2_chars.push(' ');
                    let new_part: String = s2_chars[s2_chars.len() - 2..].iter().collect();
                    mod_t2 += self.get_time(&new_part);
                }
                // 没有上文：直接返回
                if t1 == 0.0 {
                    return (s2_chars.iter().collect(), mod_t2);
                }

                let mut s1_chars: Vec<char> = s1.chars().collect();
                let mut mod_t1 = t1;

                // 新码以非空格的标点开头，且上文末尾为空格：去掉上文末尾的空格
                if s1_last == ' '
                    && s2_first != ' '
                    && !a.contains(s2_first)
                    && !n.contains(s2_first)
                {
                    if s1_chars.len() > 1 {
                        let removed_part: String = s1_chars[s1_chars.len() - 2..].iter().collect();
                        mod_t1 -= self.get_time(&removed_part);
                    }
                    s1_chars.pop().expect("无法删除头部末尾的空格。");
                }
                // 新码以形码或数字开头，且上文末尾为字母：在上文末尾加空格
                else if a.contains(s1_last) && (x.contains(s2_first) || n.contains(s2_first)) {
                    s1_chars.push(' ');
                    if s1_chars.len() > 1 {
                        let new_part: String = s1_chars[s1_chars.len() - 2..].iter().collect();
                        mod_t1 += self.get_time(&new_part);
                    }
                }

                // 连接
                let mut mid_str = String::new();
                mid_str.push(s1_chars[s1_chars.len() - 1]);
                mid_str.push(s2_chars[0]);
                let time_cost = mod_t1 + mod_t2 + self.get_time(&mid_str);
                s1_chars.append(&mut s2_chars);
                let route: String = s1_chars.iter().collect();
                (route, time_cost)
            }
            _ => panic!("未知的连接方法代号。"),
        }
    }
}
