use crate::route_connector::RouteConnector;
use std::collections::HashMap;

pub(crate) struct RouteBuffer {
    /// 编码路径缓冲区：索引为编码终点的字符位置，HashMap的键为编码，值为当量
    buffer: Vec<HashMap<String, f64>>,
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
            let buffer = vec![HashMap::new(); size];
            Ok(RouteBuffer {
                buffer,
                connector,
                head: 0,
                distance: 0,
                connected: false,
                global_best_route: (String::new(), 0.0),
            })
        }
    }

    fn clear(&mut self) {
        for i in 0..self.buffer.len() {
            self.buffer[i].clear();
        }
        self.head = 0;
        self.distance = 0;
        self.connected = false;
    }

    pub(crate) fn next(&mut self) {
        self.buffer[self.head].clear();
        self.head = (self.head + 1) % self.buffer.len();
        self.distance -= 1;
        self.connected = false;
    }

    /// 获取当前位置当量最小、码长最短的路径
    fn get_local_best_route(&self) -> (String, f64) {
        if self.buffer[self.head].is_empty() {
            (String::new(), 0.0)
        } else {
            let (code, time) = self.buffer[self.head]
                .iter()
                .min_by(|a, b| {
                    a.1.partial_cmp(&b.1)
                        .expect("路径当量中存在NaN")
                        .then(a.0.len().cmp(&b.0.len()))
                        .then(a.0.cmp(&b.0))
                })
                .expect("无法获取局部最优路径");
            (code.clone(), *time)
        }
    }

    /// 在当前位置连接编码
    pub(crate) fn connect_code(&mut self, word_len: usize, tail_code: &str, tail_time: f64) {
        // 取出当前位置的最优路径
        let mut best_route = self.get_local_best_route();

        // 如果路径太长，且当前集合就是唯一集合（全局最优路径），则暂存
        if best_route.0.len() > 1000 && self.distance == 0 {
            self.global_best_route = self.connector.connect(
                &self.global_best_route.0,
                self.global_best_route.1,
                &best_route.0,
                best_route.1,
            );
            best_route = (String::new(), 0.0);
            self.clear();
        }

        // 连接编码
        let index = (self.head + word_len) % self.buffer.len();
        let (code, time) =
            self.connector
                .connect(&best_route.0, best_route.1, tail_code, tail_time);
        self.buffer[index].insert(code, time);

        // 更新状态
        self.distance = self.distance.max(word_len);
        self.connected = true;
    }

    /// 获取是否在当前位置连接过编码
    pub(crate) fn is_connected(&self) -> bool {
        self.connected
    }

    /// 获取全局最优路径
    pub(crate) fn get_global_best_route(&self) -> Result<(String, f64), &'static str> {
        if self.buffer[self.head].is_empty() {
            Err("编码无法到达文本尾部")
        } else if self.distance != 0 {
            Err("存在超出文本尾部的编码")
        } else {
            let (code, time) = self.get_local_best_route();
            if code.is_empty() {
                Ok((self.global_best_route.0.clone(), self.global_best_route.1))
            } else {
                Ok(self.connector.connect(
                    &self.global_best_route.0,
                    self.global_best_route.1,
                    &code,
                    time,
                ))
            }
        }
    }
}
