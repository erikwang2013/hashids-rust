//! 项目宠物「哈希迪 Hashy」：字形对齐、问候输出、README 展示。

use hashids::mascot;

#[test]
fn art_is_a_well_formed_block() {
    let lines: Vec<&str> = mascot::art().lines().collect();
    assert_eq!(lines.len(), 8, "宠物应为 8 行");
    assert!(!mascot::art().ends_with('\n'), "art() 无尾随换行");

    // 脸部方块（第 3..=7 行）每行等宽。
    let widths: Vec<usize> = lines[2..7]
        .iter()
        .map(|line| line.chars().count())
        .collect();
    assert!(
        widths.windows(2).all(|pair| pair[0] == pair[1]),
        "方块未对齐: {widths:?}"
    );

    // 天线与两只脚在方块内的水平位置对齐。
    let block_start = lines[2].chars().position(|c| c == '╭').unwrap();
    let block_width = lines[2].chars().count() - block_start;
    let antenna = lines[1].chars().position(|c| c == '│').unwrap();
    assert_eq!(antenna, block_start + block_width / 2, "天线未居中");
    let foot = lines[7].chars().position(|c| c == '╵').unwrap();
    assert_eq!(foot, block_start + 3, "左脚未落在 ╰──┬ 之下");
}

#[test]
fn greet_appends_name_and_tagline() {
    let greet = mascot::greet();
    assert!(greet.starts_with(mascot::art()));
    assert!(greet.ends_with('\n'));
    assert!(greet.contains(mascot::NAME));
    assert!(greet.contains(mascot::TAGLINE));
    assert_eq!(
        greet,
        format!(
            "{}\n{} · {}\n",
            mascot::art(),
            mascot::NAME,
            mascot::TAGLINE
        )
    );
}

#[test]
fn readme_shows_the_mascot() {
    let readme = include_str!("../README.md");
    assert!(
        readme.contains(mascot::art()),
        "README 必须包含 ASCII 宠物形象"
    );
    assert!(readme.contains(mascot::NAME));
    assert!(readme.contains(mascot::TAGLINE));
}
