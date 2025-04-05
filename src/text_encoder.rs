use crate::route_buffer::RouteBuffer;
use std::collections::HashMap;

pub(crate) fn encode_text(
    text: &Vec<char>,
    dict: HashMap<char, HashMap<String, (String, f64)>>,
    buffer: RouteBuffer,
) -> (String, f64) {
}
