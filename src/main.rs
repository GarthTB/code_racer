mod code_analyzer;
mod config_loader;
mod console_reader;
mod dict_loader;
mod report_saver;
mod route_buffer;
mod route_connector;
mod text_encoder;

fn main() {
    println!("欢迎使用code_racer赛码器！");
    println!("版本号：0.3.0 (20250408)");
    println!("作者：GarthTB <g-art-h@outlook.com>");
    println!("源码：https://github.com/GarthTB/code_racer");

    fn exit_with_error<T>(message: &str) -> T {
        println!("程序异常中止！错误信息：{message}");
        console_reader::read_line();
        std::process::exit(1);
    }

    // 加载配置文件
    let layout = config_loader::load_layout().unwrap_or_else(exit_with_error);
    let punct_items = config_loader::load_punct_items().unwrap_or_else(exit_with_error);
    let time_map = config_loader::load_time_map().unwrap_or_else(exit_with_error);

    // 读取输入并加载其余配置
    let connector = console_reader::get_connector(time_map);
    let (dict, max_word_len) = console_reader::get_dict(punct_items, connector.clone());
    let text_path = console_reader::get_text_path();

    // 创建缓冲区，开始编码
    let buffer_size = 16.max(max_word_len);
    let mut buffer =
        route_buffer::RouteBuffer::new(buffer_size, connector).unwrap_or_else(exit_with_error);
    let (text_len, route, time) =
        text_encoder::encode(&text_path, dict, &mut buffer).unwrap_or_else(exit_with_error);

    // 输出报告
    let report = code_analyzer::analyze(layout, text_len, route, time);
    report_saver::save(&text_path, "最小当量编码报告", report);
    if buffer.unknown_keys_count() > 0
        && console_reader::need_to_report_unknown_keys(buffer.unknown_keys_count())
    {
        buffer.report_unknown_keys(&text_path);
    }

    // 等待用户输入
    println!("程序执行完毕。按回车键退出...");
    console_reader::read_line();
}
