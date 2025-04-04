use crate::route_connector::RouteConnector;
use std::collections::HashMap;

struct RouteBuffer {
    /// 编码路径缓冲区：索引为编码终点的字符位置，HashMap的键为编码，值为当量
    buffer: Vec<HashMap<String, f64>>,
    /// 编码路径连接器
    connector: RouteConnector,
    /// 当前位置
    head: usize,
    /// 缓冲区内最远终点位置与当前位置的距离
    distance: usize,
    /// 暂存的最优路径
    global_best_route: (String, f64),
}

impl RouteBuffer {
    fn new(size: usize, connector: RouteConnector) -> Result<Self, &'static str> {
        if size == 0 {
            Err("编码路径缓冲区大小不能为0")
        } else {
            let mut buffer = vec![HashMap::new(); size];
            buffer[0].insert(String::new(), 0.0);
            Ok(RouteBuffer {
                buffer,
                connector,
                head: 0,
                distance: 0,
                global_best_route: (String::new(), 0.0),
            })
        }
    }

    fn clear(&mut self) {
        for i in 0..self.buffer.len() {
            self.buffer[i].clear();
        }
        self.buffer[0].insert(String::new(), 0.0);
        self.head = 0;
        self.distance = 0;
    }

    fn next(&mut self) {
        self.buffer[self.head].clear();
        self.head = (self.head + 1) % self.buffer.len();
        self.distance -= 1;
    }

    /// 获取当前位置当量最小的路径
    fn get_local_best_route(&self) -> (String, f64) {
        if self.buffer[self.head].is_empty() {
            (String::new(), 0.0)
        } else {
            let (code, time) = self.buffer[self.head]
                .iter()
                .min_by(|a, b| a.1.partial_cmp(&b.1).expect("路径当量中存在NaN"))
                .expect("无法获取最优路径");
            (code.clone(), *time)
        }
    }

    /// 在当前位置连接编码
    fn connect_code(&mut self, length: usize, tail_code: &str, tail_time: f64) {
        // 取出当前位置的最优路径
        let index = (self.head + length) % self.buffer.len();
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
        let (code, time) =
            self.connector
                .connect(&best_route.0, best_route.1, tail_code, tail_time);
        self.buffer[index].insert(code, time);

        // 更新最远终点位置
        if self.distance < length {
            self.distance = length;
        }
    }

    /// 获取全局最优路径
    fn get_global_best_route(&self) -> Result<(String, f64), &'static str> {
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
