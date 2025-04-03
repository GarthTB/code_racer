use crate::route_connector::RouteConnector;
use std::collections::HashSet;

struct RouteBuffer {
    /// 编码路径缓冲区：每一项是一个HashSet，按终点位置存储编码路径
    buffer: Vec<HashSet<(String, f64)>>,
    /// 编码路径连接器
    connector: RouteConnector,
    /// 当前处理的终点位置
    head_pos: usize,
    /// 连接后编码路径的最远终点位置
    longest_tail_pos: usize,
    /// 是否在当前位置连接过编码
    encoded: bool,
}

impl RouteBuffer {
    fn new(size: usize, connector: RouteConnector) -> Result<Self, &'static str> {
        if size == 0 {
            Err("编码路径缓冲区大小不能为0")
        } else {
            let buffer = vec![HashSet::new(); size];
            buffer[0].insert(("".into_string(), 0.0));
            Ok(RouteBuffer {
                buffer,
                connector,
                head_pos: 0,
                longest_tail_pos: 0,
                encoded: false,
            })
        }
    }

    fn clear(&mut self) {
        for i in 0..self.buffer.len() {
            self.buffer[i].clear();
        }
        self.head_pos = 0;
        self.longest_tail_pos = 0;
        self.encoded = false;
        self.buffer[0].insert(("".into_string(), 0.0));
    }

    fn next(&mut self) {
        self.buffer[self.head_pos].clear();
        self.head_pos = (self.head_pos + 1) % self.buffer.len();
        self.encoded = false;
    }

    /// 没有以当前位置为终点的路径
    fn current_pos_is_empty(&self) -> bool {
        self.buffer[self.head_pos].is_empty()
    }

    /// 除了当前位置，没有以其他位置为终点的路径
    fn current_set_is_only(&self) -> bool {
        self.head_pos == self.longest_tail_pos
    }

    /// 获取当前位置当量最小的路径
    fn get_optimal_route(&self) -> &(String, f64) {
        if self.buffer[self.head_pos].is_empty() {
            &("".into_string(), 0.0)
        } else {
            self.buffer[self.head_pos]
                .iter()
                .min_by(|(_, a), (_, b)| a.partial_cmp(b).expect("出现当量为NaN的路径"))
                .expect("无法获取当量最小的路径")
        }
    }

    /// 在当前位置连接编码
    fn connect_code(&mut self, length: usize, tail_code: &str, tail_time_cost: f64) {
        // 连接并记录路径
        let index = (self.head_pos + length) % self.buffer.len();
        let (head_code, head_time_cost) = self.get_optimal_route();
        let (new_code, new_time_cost) =
            self.connector
                .connect(head_code, *head_time_cost, tail_code, tail_time_cost);
        self.buffer[index].insert((new_code, new_time_cost));
        // 更新最远终点位置
        if self.head_pos == self.longest_tail_pos // 没有连接过编码
            || (self.head_pos < self.longest_tail_pos // 最远终点没有越界
                && length > self.longest_tail_pos - self.head_pos)
            || (self.head_pos > self.longest_tail_pos // 最远终点越界
                && length > self.longest_tail_pos + self.buffer.len() - self.head_pos)
        {
            self.longest_tail_pos = index;
        }
    }
}
