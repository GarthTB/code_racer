use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub(crate) struct RouteConnector {
    /// 按键之间的用时当量：键为两个按键对应的字符，值为用时当量
    time_map: HashMap<(char, char), f64>,
    /// 找不到当量的按键组合
    unknown_keys: HashSet<(char, char)>,
    /// 连接方法代号：0-空格或符号，1-无间隔，2-键道顶功
    method_code: usize,
}

impl RouteConnector {
    pub(crate) fn new(time_map: HashMap<(char, char), f64>, method_code: usize) -> RouteConnector {
        RouteConnector {
            time_map,
            unknown_keys: HashSet::new(),
            method_code,
        }
    }

    pub(crate) fn unknown_keys_count(&self) -> usize {
        self.unknown_keys.len()
    }

    pub(crate) fn report_unknown_keys(&mut self, text_path: &PathBuf) {}

    pub(crate) fn get_time(&mut self, chars: &Vec<char>) -> f64 {
        let mut sum = 0.0;
        for i in 0..chars.len() - 1 {
            let key = (chars[i], chars[i + 1]);
            match self.time_map.get(&key) {
                Some(value) => sum += value,
                None => {
                    self.unknown_keys.insert(key);
                    sum += 1.5;
                }
            }
        }
        (sum * 1000.0).round() / 1000.0 // 避免浮点数精度问题
    }

    pub(crate) fn connect(&mut self, s1: &str, s2: &str, t1: f64, t2: f64) -> (String, f64) {
        // 取出前部的末字符、后部首字符、后部的末字符
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

        fn is_letter(c: char) -> bool {
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ".contains(c)
        }

        fn is_number(c: char) -> bool {
            "0123456789".contains(c)
        }

        // 键道形码码元
        fn is_xing(c: char) -> bool {
            "aiouvAIOUV".contains(c)
        }

        // 键道音码码元
        fn is_yin(c: char) -> bool {
            "bcdefghjklmnpqrstwxyzBCDEFGHJKLMNPQRSTWXYZ".contains(c)
        }

        match self.method_code {
            0 => {
                // 字母 + (字母或数字)，则加空格
                if s1.is_empty() {
                    (s2.to_string(), t2)
                } else if is_letter(s1_last) && (is_letter(s2_first) || is_number(s2_first)) {
                    let mut s = s1.to_string();
                    s.push(' ');
                    s.push_str(s2);
                    let mid = vec![s1_last, ' ', s2_first];
                    let t = t1 + t2 + self.get_time(&mid);
                    (s, t)
                } else {
                    let s: String = s1.chars().chain(s2.chars()).collect();
                    let mid = vec![s1_last, s2_first];
                    let t = t1 + t2 + self.get_time(&mid);
                    (s, t)
                }
            }
            1 => {
                if s1.is_empty() {
                    (s2.to_string(), t2)
                } else {
                    let s: String = s1.chars().chain(s2.chars()).collect();
                    let mid = vec![s1_last, s2_first];
                    let t = t1 + t2 + self.get_time(&mid);
                    (s, t)
                }
            }
            2 => {
                let mut s2_chars: Vec<char> = s2.chars().collect();
                let mut mod_t2 = t2;

                // 新码以音码结尾，且不足4码：新码后补空格
                if is_yin(s2_last) && s2_chars.len() < 4 {
                    s2_chars.push(' ');
                    let tail = vec![s2_last, ' '];
                    mod_t2 += self.get_time(&tail);
                }
                // 没有上文：直接返回
                if s1.is_empty() {
                    return (s2_chars.into_iter().collect(), mod_t2);
                }

                let mut s1_chars: Vec<char> = s1.chars().collect();
                let mut mod_t1 = t1;

                // 上文末尾为空格，且新码以非空格的标点开头：去掉上文末尾的空格
                if s1_last == ' ' && s2_first != ' ' && !is_letter(s2_first) && !is_number(s2_first)
                {
                    s1_chars.pop().expect("无法删除头部末尾的空格。");
                    if s1_chars.len() > 0 {
                        let tail = vec![s1_chars[s1_chars.len() - 1], ' '];
                        mod_t1 -= self.get_time(&tail);
                    }
                }
                // 上文末尾为字母，且新码以形码或数字开头：在上文末尾加空格
                else if is_letter(s1_last) && (is_xing(s2_first) || is_number(s2_first)) {
                    s1_chars.push(' ');
                    let tail = vec![s1_last, ' '];
                    mod_t1 += self.get_time(&tail);
                }

                // 连接
                let mid = vec![s1_chars[s1_chars.len() - 1], s2_chars[0]];
                let time = mod_t1 + mod_t2 + self.get_time(&mid);
                s1_chars.append(&mut s2_chars);
                (s1_chars.into_iter().collect(), time)
            }
            _ => panic!("未知的连接方法代号。"),
        }
    }
}
