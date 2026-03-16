use clap::Parser;
use gbk2utf8::{run, Config};
use std::process;

mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

fn main() {
    let config = Config::parse();

    println!(
        "版本 {}，编译于 [{}]，由 {} 构建（目标: {}）",
        built_info::PKG_VERSION,
        built_info::BUILT_TIME_UTC,
        built_info::RUSTC_VERSION,
        built_info::TARGET
    );

    let result = match run(&config) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("❌ 扫描目录失败: {}", e);
            process::exit(1);
        }
    };

    if !result.errors.is_empty() {
        println!("\n以下文件转换失败：");
        for (path, err) in &result.errors {
            println!("{}: {}", path.display(), err);
        }
        process::exit(2);
    } else {
        println!("✅ 所有文件处理完成");
    }

    println!(
        "\n统计信息:\n1.成功转换: {}\n2.转换失败: {}\n3.无需转换: {}",
        result.stats.converted, result.stats.failed, result.stats.no_conversion
    );
}
