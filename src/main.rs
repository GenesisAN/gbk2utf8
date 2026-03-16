use clap::Parser;
use gbk2utf8::{run, Config, UiLang};
use std::process;

mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

fn main() {
    let config = Config::parse();
    let is_zh = matches!(config.ui_lang(), UiLang::Zh);

    if is_zh {
        println!(
            "版本 {}，编译于 [{}]，由 {} 构建（目标: {}）",
            built_info::PKG_VERSION,
            built_info::BUILT_TIME_UTC,
            built_info::RUSTC_VERSION,
            built_info::TARGET
        );
    } else {
        println!(
            "version {}, built at [{}], built by {} (target: {})",
            built_info::PKG_VERSION,
            built_info::BUILT_TIME_UTC,
            built_info::RUSTC_VERSION,
            built_info::TARGET
        );
    }

    let result = match run(&config) {
        Ok(result) => result,
        Err(e) => {
            if is_zh {
                eprintln!("❌ 扫描目录失败: {}", e);
            } else {
                eprintln!("❌ failed to scan directory: {}", e);
            }
            process::exit(1);
        }
    };

    if !result.errors.is_empty() {
        if is_zh {
            println!("\n以下文件转换失败：");
        } else {
            println!("\nfailed to convert these files:");
        }
        for (path, err) in &result.errors {
            println!("{}: {}", path.display(), err);
        }
        process::exit(2);
    } else {
        if is_zh {
            println!("✅ 所有文件处理完成");
        } else {
            println!("✅ all files processed");
        }
    }

    if is_zh {
        println!(
            "\n统计信息:\n1.成功转换: {}\n2.转换失败: {}\n3.无需转换: {}",
            result.stats.converted, result.stats.failed, result.stats.no_conversion
        );
    } else {
        println!(
            "\nsummary:\n1.converted: {}\n2.failed: {}\n3.no conversion needed: {}",
            result.stats.converted, result.stats.failed, result.stats.no_conversion
        );
    }
}
