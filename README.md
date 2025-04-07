# [code_racer 🐎 赛码器](https://github.com/GarthTB/code_racer)

[![Language](https://img.shields.io/badge/Built%20with-Rust-brown)](https://www.rust-lang.org/zh-CN/)
[![Version](https://img.shields.io/badge/Latest%20Release-0.3.0-brightgreen)](https://github.com/GarthTB/code_racer/releases)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue)](https://www.apache.org/licenses/LICENSE-2.0)

轻松计算数十万字的文本在自定义的键盘布局下、用特定输入法时，击键时间当量[1]最小的打法，并简单分析这个最优的编码。

## 配置文件

### layout.txt

- 定义键盘布局，用于统计。共14行。每一行分别为：

```
[数字排的码元]
[上排的码元]
[中排的码元]
[下排的码元]
[底排的码元]
[左手小指的码元]
[左手无名指的码元]
[左手中指的码元]
[左手食指的码元]
[右手食指的码元]
[右手中指的码元]
[右手无名指的码元]
[右手小指的码元]
[拇指的码元]
```

### punct_dict.txt

- 定义标点符号的按键打法，格式和内部加载方法与词库完全相同。但无论优先级多大，编码始终排在词库中的编码之后。

- 每行格式为`标点符号\t编码[\t优先级]`，用`#`号引导注释。

### time_cost.txt

- 定义两个键连着按下之间的相对用时，称之为当量[1]。定义最快的组合为1.0。

- 每行格式为`两个键对应的编码\t当量`

## 注意

- 词库每行格式为`标点符号\t编码[\t优先级]`，用`#`号引导注释。
- 默认的当量文件中没有shift键（默认编码为↑）和退格键（默认编码为←），所以控制台会出现找不到当量的报告。最终路径不会受此影响，可以忽略。
- 词库在载入过程中自动计算选重和翻页键。词库中的条目依次按`优先级降序、码长升序、词升序、码升序`来争夺码位。词库文件本身的条目顺序无效。
- 码位被占用不代表这个打法会被使用。有多个编码的词，永远只会使用当量最小的编码。
- 分析报告中，`偏倚率 = 100% * (左右手键数的差 / 左右手键数的和)`
- 分析报告中，`互击率 = 100% * (左右左 + 右左右) / (总码数 - 2)`

## 引用

- [1]陈一凡,张鹿,周志农.键位相关速度当量的研究[J].中文信息学报,1990,(04):12-18+11.

## 发布日志

### v0.3.0 - 20250408

- 新增：不直接打印，而是在统计完成后选择性保存找不到当量的按键组合
- 修复：无法生成正确选重键的问题
- 修复：键道顶功的标点顶功算法

### v0.2.0 - 20250406

- 修复：跳过部分编码的问题
- 修复：连击数统计错乱的问题
- 修复：标点符号配置文件错误
- 优化：略微提升性能

### v0.1.0 - 20250406

- 首个发布！