use std::collections::HashMap;

struct RouteConnector {
    /// 按键之间的用时当量，键为两个按键，值为用时
    time_cost_map: HashMap<String, f64>,
    /// 连接方法代号：0-空格或符号，1-无间隔，2-键道顶功
    method_code: usize,
}

impl RouteConnector {
    fn new(time_cost_map: HashMap<String, f64>, method_code: usize) -> RouteConnector {
        RouteConnector {
            time_cost_map,
            method_code,
        }
    }

    fn get_time_cost(&self, text: &str) -> f64 {
        let mut total = 0.0;
        let mut chars = text.chars().peekable();
        let mut key = String::with_capacity(2);

        while let Some(c1) = chars.next() {
            if let Some(&c2) = chars.peek() {
                key.clear();
                key.push(c1);
                key.push(c2);

                match self.time_cost_map.get(&key) {
                    Some(value) => total += value,
                    None => {
                        println!("找不到{key}对应的当量。已默认为1.4。");
                        total += 1.4;
                    }
                }
            }
        }

        total
    }

    fn connect(
        &self,
        head_code: &str,
        head_time_cost: f64,
        tail_code: &str,
        tail_time_cost: f64,
    ) -> (String, f64) {
        let letters = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let numbers = "0123456789";

        let connect_with = |mid_chars: &str, head_last: char, tail_first: char| {
            let mut route = String::new();
            route.push_str(head_code);
            route.push_str(mid_chars);
            route.push_str(tail_code);
            let mut mid_str = String::new();
            mid_str.push(head_last);
            mid_str.push_str(mid_chars);
            mid_str.push(tail_first);
            let time_cost = head_time_cost + tail_time_cost + self.get_time_cost(&mid_str);
            (route, time_cost)
        };

        match self.method_code {
            0 => {
                if head_code.is_empty() {
                    (tail_code.into_string(), tail_time_cost)
                } else {
                    let head_last = head_code
                        .chars()
                        .next_back()
                        .expect("无法获取头部末个字符。");
                    let tail_first = tail_code.chars().next().expect("无法获取尾部首个字符。");
                    // 字母 + (字母或数字)，则加空格
                    match letters.contains(head_last)
                        && (letters.contains(tail_first) || numbers.contains(tail_first))
                    {
                        true => connect_with(" ", head_last, tail_first),
                        false => connect_with("", head_last, tail_first),
                    }
                }
            }
            1 => {
                let head_last = head_code
                    .chars()
                    .next_back()
                    .expect("无法获取头部末个字符。");
                let tail_first = tail_code.chars().next().expect("无法获取尾部首个字符。");
                connect_with("", head_last, tail_first)
            }
            2 => {
                let xing = "aiouv"; // 形码码元
                let yin = "bcdefghjklmnpqrstwxyz"; // 音码码元
                let mut head_chars: Vec<char> = head_code.chars().collect();
                let mut tail_chars: Vec<char> = tail_code.chars().collect();
                let mut real_head_time_cost = head_time_cost;
                let mut real_tail_time_cost = tail_time_cost;

                // 新码不足4码，且以音码结尾：后补空格
                if tail_chars.len() < 4 && yin.contains(tail_chars[tail_chars.len() - 1]) {
                    let mut space = String::new();
                    space.push(tail_chars[tail_chars.len() - 1]);
                    space.push(' ');
                    real_tail_time_cost += self.get_time_cost(&space);
                    tail_chars.push(' ');
                }
                // 没有上文：直接返回
                if head_chars.len() == 0 {
                    return (tail_chars.iter().collect(), real_tail_time_cost);
                }

                // 新码以非空格的标点开头，且前为空格：去掉空格
                if head_chars[head_chars.len() - 1] == ' '
                    && tail_chars[0] != ' '
                    && !letters.contains(tail_chars[0])
                    && !numbers.contains(tail_chars[0])
                {
                    let mut space = String::new();
                    let head_last = head_chars.pop().expect("无法获取头部末个字符。");
                    space.push(head_last);
                    space.push(' ');
                    real_head_time_cost -= self.get_time_cost(&space);
                }
                // 新码以形码或数字开头，且前为字母：前加空格
                else if letters.contains(head_chars[head_chars.len() - 1])
                    && (xing.contains(tail_chars[0]) || numbers.contains(tail_chars[0]))
                {
                    let mut space = String::new();
                    space.push(head_chars[head_chars.len() - 1]);
                    space.push(' ');
                    real_head_time_cost += self.get_time_cost(&space);
                    head_chars.push(' ');
                }

                // 连接
                let mut route = String::new();
                route.push_str(&head_chars.iter().collect::<String>());
                route.push_str(&tail_chars.iter().collect::<String>());
                let mut mid_str = String::new();
                mid_str.push(head_chars[head_chars.len() - 1]);
                mid_str.push(tail_chars[0]);
                let time_cost =
                    real_head_time_cost + real_tail_time_cost + self.get_time_cost(&mid_str);
                (route, time_cost)
            }
            _ => panic!("未知的连接方法代号。"),
        }
    }
}
