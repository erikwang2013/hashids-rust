//! 项目宠物「哈希迪 Hashy」。
//!
//! 图形本体见 `docs/mascot.svg`；终端里没有 SVG，因此这里保留一份等价的
//! ASCII 版本（与 PHP 版 `Erikwang2013\Hashids\Mascot` 逐字符一致）。

/// 宠物名。
pub const NAME: &str = "哈希迪 Hashy";

/// 一句话标语。
pub const TAGLINE: &str = "把数据库自增 ID 换成短小、不可猜测的字符串";

/// 头顶天线、胸口带 `#` 的圆角方块小宠物（无尾随换行）。
pub fn art() -> &'static str {
    concat!(
        "           ●\n",
        "           │\n",
        "      ╭─────────╮\n",
        "      │ ◉     ◉ │\n",
        "      │    ‿    │\n",
        "      │    #    │\n",
        "      ╰──┬───┬──╯\n",
        "         ╵   ╵",
    )
}

/// 可直接打印 / 写日志的完整问候，末尾带换行。
pub fn greet() -> String {
    format!("{}\n{} · {}\n", art(), NAME, TAGLINE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn art_aligns_to_same_width() {
        let lines: Vec<&str> = art().lines().collect();
        assert_eq!(lines.len(), 8);
        // 笑脸块（第 3-7 行）与天线首行应保持方块对齐。
        assert_eq!(lines[0].chars().count(), 12);
        assert!(!art().ends_with('\n'));
    }

    #[test]
    fn greet_appends_name_and_tagline() {
        assert_eq!(greet(), format!("{}\n哈希迪 Hashy · {}\n", art(), TAGLINE));
    }
}
