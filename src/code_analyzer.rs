use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};

pub(crate) fn analyze(
    layout: Vec<String>,
    text_len: usize,
    route: String,
    time: f64,
) -> Vec<String> {
    // 简单分析
    let chars: Vec<char> = route.chars().collect();
    let code_len = chars.len() as f64 / text_len as f64;
    let time_per_char = time / text_len as f64;
    let time_per_key = time / chars.len() as f64;

    // 简单返回
    if layout.len() != 14 {
        println!("键盘布局配置错误，将只进行简单分析。");
        return vec![
            route,
            "---以上为最优编码路径，以下为简单分析结果---".to_string(),
            format!("字数\t{}", text_len),
            format!("码数\t{}", chars.len()),
            format!("当量\t{:.1}", time),
            format!("字均码长\t{:.8}", code_len),
            format!("字均当量\t{:.4}", time_per_char),
            format!("码均当量\t{:.4}", time_per_key),
        ];
    }

    // 完整分析的变量和方法
    let mut parts_count = Vec::with_capacity(14); // 每组码的计数
    for _ in 0..14 {
        parts_count.push(AtomicUsize::new(0));
    }
    let s_leap_count = AtomicUsize::new(0); // 同指跨1排
    let m_leap_count = AtomicUsize::new(0); // 同指跨2排
    let l_leap_count = AtomicUsize::new(0); // 同指跨3排
    let double_count = AtomicUsize::new(0); // 同键按2次
    let triple_count = AtomicUsize::new(0); // 同键按3次
    let quadruple_count = AtomicUsize::new(0); // 同键按4次
    let quintuple_count = AtomicUsize::new(0); // 同键按5次
    let turns_count = AtomicUsize::new(0); // 左右左 + 右左右的次数

    let double_contains = |c1: char, c2: char, s1: &str, s2: &str| {
        (s1.contains(c1) && s2.contains(c2)) || (s1.contains(c2) && s2.contains(c1))
    };

    let same_finger = |c1: char, c2: char| {
        double_contains(c1, c2, &layout[5], &layout[5])
            || double_contains(c1, c2, &layout[6], &layout[6])
            || double_contains(c1, c2, &layout[7], &layout[7])
            || double_contains(c1, c2, &layout[8], &layout[8])
            || double_contains(c1, c2, &layout[9], &layout[9])
            || double_contains(c1, c2, &layout[10], &layout[10])
            || double_contains(c1, c2, &layout[11], &layout[11])
            || double_contains(c1, c2, &layout[12], &layout[12])
    };

    let s_leap = |c1: char, c2: char| {
        double_contains(c1, c2, &layout[0], &layout[1])
            || double_contains(c1, c2, &layout[1], &layout[2])
            || double_contains(c1, c2, &layout[2], &layout[3])
    };

    let m_leap = |c1: char, c2: char| {
        double_contains(c1, c2, &layout[0], &layout[2])
            || double_contains(c1, c2, &layout[1], &layout[3])
    };

    let l_leap = |c1: char, c2: char| double_contains(c1, c2, &layout[0], &layout[3]);

    let left_keys: HashSet<char> = layout[5]
        .chars()
        .chain(layout[6].chars())
        .chain(layout[7].chars())
        .chain(layout[8].chars())
        .collect();

    let right_keys: HashSet<char> = layout[9]
        .chars()
        .chain(layout[10].chars())
        .chain(layout[11].chars())
        .chain(layout[12].chars())
        .collect();

    let turns = |c1: char, c2: char, c3: char| {
        left_keys.contains(&c1) && right_keys.contains(&c2) && left_keys.contains(&c3)
            || right_keys.contains(&c1) && left_keys.contains(&c2) && right_keys.contains(&c3)
    };

    let count_1_char = |c: char| {
        for i in 0..14 {
            if layout[i].contains(c) {
                parts_count[i].fetch_add(1, Ordering::Relaxed);
            }
        }
    };

    let count_2_chars = |c1: char, c2: char| {
        if c1 == c2 {
            double_count.fetch_add(1, Ordering::Relaxed);
        } else if same_finger(c1, c2) {
            if s_leap(c1, c2) {
                s_leap_count.fetch_add(1, Ordering::Relaxed);
            } else if m_leap(c1, c2) {
                m_leap_count.fetch_add(1, Ordering::Relaxed);
            } else if l_leap(c1, c2) {
                l_leap_count.fetch_add(1, Ordering::Relaxed);
            }
        }
    };

    let count_3_chars = |c1: char, c2: char, c3: char| {
        if c1 == c2 && c2 == c3 {
            triple_count.fetch_add(1, Ordering::Relaxed);
        } else if turns(c1, c2, c3) {
            turns_count.fetch_add(1, Ordering::Relaxed);
        }
    };

    // 开始并行分析
    println!("并行分析编码...");
    (0..chars.len()).into_par_iter().for_each(|i| {
        count_1_char(chars[i]);
        if i > 0 {
            count_2_chars(chars[i - 1], chars[i]);
        }
        if i > 1 {
            count_3_chars(chars[i - 2], chars[i - 1], chars[i]);
        }
        if i > 2
            && chars[i - 3] == chars[i - 2]
            && chars[i - 2] == chars[i - 1]
            && chars[i - 1] == chars[i]
        {
            quadruple_count.fetch_add(1, Ordering::Relaxed);
        }
        if i > 3
            && chars[i - 4] == chars[i - 3]
            && chars[i - 3] == chars[i - 2]
            && chars[i - 2] == chars[i - 1]
            && chars[i - 1] == chars[i]
        {
            quintuple_count.fetch_add(1, Ordering::Relaxed);
        }
    });
    println!("分析完成。");

    // 完整返回
    let left_count = parts_count[5].load(Ordering::Relaxed)
        + parts_count[6].load(Ordering::Relaxed)
        + parts_count[7].load(Ordering::Relaxed)
        + parts_count[8].load(Ordering::Relaxed);

    let right_count = parts_count[9].load(Ordering::Relaxed)
        + parts_count[10].load(Ordering::Relaxed)
        + parts_count[11].load(Ordering::Relaxed)
        + parts_count[12].load(Ordering::Relaxed);

    let mut real_double_count = double_count.load(Ordering::Relaxed);
    let mut real_triple_count = triple_count.load(Ordering::Relaxed);
    let mut real_quadruple_count = quadruple_count.load(Ordering::Relaxed);
    let real_quintuple_count = quintuple_count.load(Ordering::Relaxed);
    real_double_count -= real_triple_count;
    real_triple_count -= real_quadruple_count;
    real_quadruple_count -= real_quintuple_count;

    let gen_deviation_report = if left_count + right_count == 0 {
        "双手键数之和为0，无法计算偏倚率。".to_string()
    } else {
        format!(
            "偏倚率\t{:.3}%",
            100.0 * (left_count as f64 - right_count as f64) / (left_count + right_count) as f64
        )
    };

    let gen_report = |name: &str, involved_len: usize, count: usize| {
        format!(
            "{name}\t{count}\t{:.3}%",
            100.0 * count as f64 / (chars.len() - involved_len + 1) as f64
        )
    };

    vec![
        route,
        "---以上为最优编码路径，以下为完整分析结果---".to_string(),
        format!("字数\t{}", text_len),
        format!("码数\t{}", chars.len()),
        format!("当量\t{:.1}", time),
        format!("字均码长\t{:.8}", code_len),
        format!("字均当量\t{:.4}", time_per_char),
        format!("码均当量\t{:.4}", time_per_key),
        gen_report("总左手", 1, left_count),
        gen_report("总右手", 1, right_count),
        gen_deviation_report,
        gen_report("数排", 1, parts_count[0].load(Ordering::Relaxed)),
        gen_report("上排", 1, parts_count[1].load(Ordering::Relaxed)),
        gen_report("中排", 1, parts_count[2].load(Ordering::Relaxed)),
        gen_report("下排", 1, parts_count[3].load(Ordering::Relaxed)),
        gen_report("底排", 1, parts_count[4].load(Ordering::Relaxed)),
        gen_report("左小指", 1, parts_count[5].load(Ordering::Relaxed)),
        gen_report("左无名", 1, parts_count[6].load(Ordering::Relaxed)),
        gen_report("左中指", 1, parts_count[7].load(Ordering::Relaxed)),
        gen_report("左食指", 1, parts_count[8].load(Ordering::Relaxed)),
        gen_report("右食指", 1, parts_count[9].load(Ordering::Relaxed)),
        gen_report("右中指", 1, parts_count[10].load(Ordering::Relaxed)),
        gen_report("右无名", 1, parts_count[11].load(Ordering::Relaxed)),
        gen_report("右小指", 1, parts_count[12].load(Ordering::Relaxed)),
        gen_report("拇指键", 1, parts_count[13].load(Ordering::Relaxed)),
        gen_report("同指跨1排", 2, s_leap_count.load(Ordering::Relaxed)),
        gen_report("同指跨2排", 2, m_leap_count.load(Ordering::Relaxed)),
        gen_report("同指跨3排", 2, l_leap_count.load(Ordering::Relaxed)),
        gen_report("两连击", 2, real_double_count),
        gen_report("三连击", 3, real_triple_count),
        gen_report("四连击", 4, real_quadruple_count),
        format!("更多连击\t{real_quintuple_count}"),
        gen_report("左右互击", 3, turns_count.load(Ordering::Relaxed)),
    ]
}
