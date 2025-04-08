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
    pub(crate) fn new(time_map: HashMap<(char, char), f64>, method_code: usize) -> Self {
        Self {
            time_map,
            unknown_keys: HashSet::new(),
            method_code,
        }
    }

    pub(crate) fn clone(&self) -> Self {
        Self {
            time_map: self.time_map.clone(),
            unknown_keys: self.unknown_keys.clone(),
            method_code: self.method_code,
        }
    }

    pub(crate) fn unknown_keys_count(&self) -> usize {
        self.unknown_keys.len()
    }

    pub(crate) fn report_unknown_keys(&self, text_path: &PathBuf) {
        let content = self
            .unknown_keys
            .iter()
            .map(|(c1, c2)| format!("{c1}{c2}"))
            .collect();
        crate::report_saver::save(text_path, "找不到当量的按键组合", content);
    }

    pub(crate) fn get_time(&mut self, chars: &[char]) -> f64 {
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
        (sum * 100.0).round() / 100.0 // 避免浮点数精度问题
    }

    pub(crate) fn connect(
        &mut self,
        s1: &[char],
        s2: &[char],
        t1: f64,
        t2: f64,
    ) -> (Vec<char>, f64) {
        // 取出前部的末字符、后部的首字符、后部的末字符
        let s1_last = s1.last().unwrap_or(&'\0');
        let s2_first = s2.first().unwrap_or(&'\0');
        let s2_last = s2.last().unwrap_or(&'\0');

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
                    (s2.to_vec(), t2)
                } else if is_letter(*s1_last) && (is_letter(*s2_first) || is_number(*s2_first)) {
                    let mid = vec![*s1_last, ' ', *s2_first];
                    let t = t1 + t2 + self.get_time(&mid);
                    let mut s = s1.to_vec();
                    s.push(' ');
                    s.append(&mut s2.to_vec());
                    (s, t)
                } else {
                    let mid = vec![*s1_last, *s2_first];
                    let t = t1 + t2 + self.get_time(&mid);
                    let mut s = s1.to_vec();
                    s.append(&mut s2.to_vec());
                    (s, t)
                }
            }
            1 => {
                if s1.is_empty() {
                    (s2.to_vec(), t2)
                } else {
                    let mid = vec![*s1_last, *s2_first];
                    let t = t1 + t2 + self.get_time(&mid);
                    let mut s = s1.to_vec();
                    s.append(&mut s2.to_vec());
                    (s, t)
                }
            }
            2 => {
                let mut mod_s2 = s2.to_vec();
                let mut mod_t2 = t2;

                // 新码以音码结尾，且不足4码：新码后补空格
                if is_yin(*s2_last) && s2.len() < 4 {
                    mod_s2.push(' ');
                    mod_t2 += self.get_time(&mod_s2[mod_s2.len() - 2..]);
                }
                // 没有上文：直接返回
                if s1.is_empty() {
                    return (s2.to_vec(), mod_t2);
                }

                let mut mod_s1 = s1.to_vec();
                let mut mod_t1 = t1;

                // 上文末尾为音码 + 空格，且新码以非空格的标点开头：去掉上文末尾的空格
                if s1.len() > 1
                    && is_yin(s1[s1.len() - 2])
                    && *s1_last == ' '
                    && *s2_first != ' '
                    && !is_letter(*s2_first)
                    && !is_number(*s2_first)
                {
                    mod_s1.pop().expect("无法删除头部末尾的空格");
                    mod_t1 -= self.get_time(&s1[s1.len() - 2..]);
                }
                // 上文末尾为字母，且新码以形码或数字开头：在上文末尾加空格
                else if is_letter(*s1_last) && (is_xing(*s2_first) || is_number(*s2_first)) {
                    mod_s1.push(' ');
                    mod_t1 += self.get_time(&mod_s1[mod_s1.len() - 2..]);
                }

                // 连接
                let mid = vec![
                    *mod_s1.last().expect("无法获取头部末字符"),
                    *mod_s2.first().expect("无法获取尾部首字符"),
                ];
                let time = mod_t1 + mod_t2 + self.get_time(&mid);
                mod_s1.append(&mut mod_s2);
                (mod_s1, time)
            }
            _ => panic!("未知的连接方法代号"),
        }
    }
}
