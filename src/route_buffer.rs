use crate::route_connector::RouteConnector;
use std::path::PathBuf;

/// 编码路径缓冲区
pub(crate) struct RouteBuffer {
    /// 索引为待编码的第一个字符的位置
    buffer: Vec<(String, f64)>,
    /// 编码路径连接器
    connector: RouteConnector,
    /// 当前位置
    head: usize,
    /// 缓冲区内最远终点位置与当前位置的距离
    distance: usize,
    /// 是否在当前位置连接过编码
    connected: bool,
    /// 暂存的最优路径
    global_best_route: (String, f64),
}

impl RouteBuffer {
    pub(crate) fn new(size: usize, connector: RouteConnector) -> Result<Self, &'static str> {
        if size == 0 {
            Err("编码路径缓冲区大小不能为0")
        } else {
            Ok(RouteBuffer {
                buffer: vec![(String::new(), 0.0); size],
                connector,
                head: 0,
                distance: 0,
                connected: false,
                global_best_route: (String::new(), 0.0),
            })
        }
    }

    /// 获取是否在当前位置连接过编码
    pub(crate) fn is_connected(&self) -> bool {
        self.connected
    }

    /// 找不到当量的按键组合数量
    pub(crate) fn unknown_keys_count(&self) -> usize {
        self.connector.unknown_keys_count()
    }

    /// 导出找不到当量的按键组合
    pub(crate) fn report_unknown_keys(&mut self, text_path: &PathBuf) {
        self.connector.report_unknown_keys(text_path);
    }

    pub(crate) fn next(&mut self) {
        self.buffer[self.head].0.clear();
        self.buffer[self.head].1 = 0.0;
        self.head = (self.head + 1) % self.buffer.len();
        self.distance -= 1;
        self.connected = false;
    }

    fn clear(&mut self) {
        for i in 0..self.buffer.len() {
            self.buffer[i].0.clear();
            self.buffer[i].1 = 0.0;
        }
        self.distance = 0;
        self.connected = false;
    }

    /// 在当前位置连接编码
    pub(crate) fn connect_code(&mut self, word_len: usize, tail_code: &str, tail_time: f64) {
        // 取出到当前位置的最优路径
        let mut best_route = self.buffer[self.head].clone();

        // 如果路径太长，且当前路径就是唯一路径（全局最优），则暂存并清空缓冲区
        if best_route.0.len() > 1000 && self.distance == 0 {
            self.global_best_route = self.connector.connect(
                &self.global_best_route.0,
                &best_route.0,
                self.global_best_route.1,
                best_route.1,
            );
            best_route = (String::new(), 0.0);
            self.clear();
        }

        // 连接编码
        let index = (self.head + word_len) % self.buffer.len();
        let (code, time) =
            self.connector
                .connect(&best_route.0, tail_code, best_route.1, tail_time);

        // 更新最优路径：未编码过，或当量更小，或同当量且编码更短
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

    pub(crate) fn get_global_best_route(&mut self) -> Result<(String, f64), &'static str> {
        if self.distance != 0 {
            Err("存在超出文本尾部的编码")
        } else {
            let best_route = self.buffer[self.head].clone();
            if best_route.0.is_empty() {
                Ok((self.global_best_route.0.clone(), self.global_best_route.1))
            } else {
                Ok(self.connector.connect(
                    &self.global_best_route.0,
                    &best_route.0,
                    self.global_best_route.1,
                    best_route.1,
                ))
            }
        }
    }
}
