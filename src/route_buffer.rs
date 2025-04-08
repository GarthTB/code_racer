use crate::route_connector::RouteConnector;
use std::path::PathBuf;

pub(crate) struct RouteBuffer {
    /// 索引为待编码的第一个字符的位置
    buffer: Vec<(Vec<char>, f64)>,
    /// 编码路径连接器
    connector: RouteConnector,
    /// 当前位置
    head: usize,
    /// 字数计数器
    count: usize,
    /// 缓冲区内最远终点位置与当前位置的距离
    distance: usize,
    /// 是否在当前位置连接过编码
    connected: bool,
    /// 暂存的全局最优路径
    global_best_route: Vec<char>,
}

impl RouteBuffer {
    pub(crate) fn new(size: usize, connector: RouteConnector) -> Result<Self, &'static str> {
        if size == 0 {
            Err("编码路径缓冲区大小不能为0")
        } else {
            Ok(Self {
                buffer: vec![(Vec::new(), 0.0); size],
                connector,
                head: 0,
                count: 0,
                distance: 0,
                connected: false,
                global_best_route: Vec::new(),
            })
        }
    }

    /// 获取是否在当前位置连接过编码
    pub(crate) fn is_connected(&self) -> bool {
        self.connected
    }

    /// 获取迭代过的字数
    pub(crate) fn count(&self) -> usize {
        self.count
    }

    /// 找不到当量的按键组合数量
    pub(crate) fn unknown_keys_count(&self) -> usize {
        self.connector.unknown_keys_count()
    }

    /// 导出找不到当量的按键组合
    pub(crate) fn report_unknown_keys(&self, text_path: &PathBuf) {
        self.connector.report_unknown_keys(text_path);
    }

    pub(crate) fn next(&mut self) {
        self.buffer[self.head].0.clear();
        self.buffer[self.head].1 = 0.0;
        self.head = (self.head + 1) % self.buffer.len();
        self.count += 1;
        self.distance -= 1;
        self.connected = false;
    }

    /// 在当前位置连接编码
    pub(crate) fn connect_code(&mut self, word_len: usize, tail_code: &[char], tail_time: f64) {
        // 如果当前路径太长，且当前路径为唯一路径（全局最优），则暂存并清空缓冲区
        if self.buffer[self.head].0.len() > 100 && self.distance == 0 {
            // 分成已经不影响后续编码的头部和仍在影响后续编码的尾部
            let (dead_part, live_part) = self.buffer[self.head].0.split_at(96);
            // 头部存入全局最优路径，尾部放回缓冲区
            self.global_best_route.append(&mut dead_part.to_vec());
            self.buffer[self.head].0 = live_part.to_vec();
        }

        // 连接编码
        let (code, time) = self.connector.connect(
            &self.buffer[self.head].0,
            tail_code,
            self.buffer[self.head].1,
            tail_time,
        );

        // 若目标位置没有编码，或当量更小，或同当量且编码更短：更新最优路径
        let index = (self.head + word_len) % self.buffer.len();
        if self.buffer[index].0.is_empty()
            || time < self.buffer[index].1
            || code.len() < self.buffer[index].0.len()
        {
            self.buffer[index] = (code, time);
        }

        // 更新状态
        self.distance = self.distance.max(word_len);
        self.connected = true;
    }

    pub(crate) fn get_global_best_route(&mut self) -> Result<(Vec<char>, f64), &'static str> {
        if self.distance != 0 {
            Err("存在超出文本尾部的编码")
        } else {
            self.global_best_route
                .append(&mut self.buffer[self.head].0.clone());
            Ok((self.global_best_route.clone(), self.buffer[self.head].1))
        }
    }
}
