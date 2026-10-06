//! 打印项目宠物「哈希迪 Hashy」的问候。
//!
//! ```bash
//! cargo run --example mascot
//! ```

fn main() {
    println!("{}", hashids::mascot::greet());
}
