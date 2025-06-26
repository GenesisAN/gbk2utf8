use encoding::all::GBK;
use encoding::{DecoderTrap, Encoding};
use std::collections::HashMap;
use std::{env, fs};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use clap::Parser;

/// GBK转UTF-8工具
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Config {
    /// 扫描的目录
    #[arg(short = 'd', long, default_value = "./", help = "要扫描的目录路径")]
    dir: String,

    /// 最少中文字符数才进行转换（GBK编码计数）
    #[arg(short = 't', long, default_value_t = 4, help = "若检测到的中文字符数量达到该值，则进行转换")]
    threshold: usize,

    /// 打印详细信息
    #[arg(short = 'i', long = "show-info", help = "显示详细的文件处理信息")]
    show_info: bool,

    /// 启用扫描模式，仅检测，不做修改
    #[arg(short = 's', long = "scan-only", help = "扫描模式：只检测，不转换文件")]
    scan_only: bool,

    /// 要求的最小连续中文字符数
    #[arg(short = 'm', long = "min-consecutive", default_value_t = 2, help = "要求的最小连续中文字符数")]
    min_consecutive: usize,

    /// 是否在转换前备份原始文件
    #[arg(short = 'b', long = "backup", help = "转换前备份原始文件")]
    backup: bool,

    /// 要检查的文件扩展名，逗号分隔
    #[arg(short = 'e', long = "extensions", value_delimiter = ',', default_value = "c,h", help = "要检查的文件扩展名，逗号分隔，例如：c,h,cpp")]
    extensions: Vec<String>,
}


// The file `built.rs` was placed there by cargo and `build.rs`
mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

fn is_chinese(c: char) -> bool {
    (c >= '\u{4e00}' && c <= '\u{9fa5}') || (c >= '\u{9fa6}' && c <= '\u{9fcb}')
}

fn contains_chinese_utf8(content: &[u8]) -> bool {
    let content_str = match std::str::from_utf8(content) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let mut chinese_count = 0;
    for c in content_str.chars() {
        if is_chinese(c) {
            chinese_count += 1;
            if chinese_count >= 4 {
                return true;
            }
        }
    }
    false
}

/// 统计GBK编码下的中文字符总数
fn count_chinese_gbk(content: &[u8]) -> usize {
    let mut count = 0;
    for i in 0..content.len().saturating_sub(1) {
        if content[i] >= 0xB0 && content[i] <= 0xF7 &&
           content[i + 1] >= 0xA1 && content[i + 1] <= 0xFE {
            count += 1;
        }
    }
    count
}

/// 统计最长连续的GBK中文字符数
fn max_consecutive_gbk_chinese(content: &[u8]) -> usize {
    let mut max_streak = 0;
    let mut current_streak = 0;

    let mut i = 0;
    while i < content.len() - 1 {
        if content[i] >= 0xB0 && content[i] <= 0xF7 &&
           content[i + 1] >= 0xA1 && content[i + 1] <= 0xFE {
            current_streak += 1;
            if current_streak > max_streak {
                max_streak = current_streak;
            }
            i += 2;
        } else {
            current_streak = 0;
            i += 1;
        }
    }

    max_streak
}

/// 扫描文件是否是GBK，并返回(总中文数, 最大连续数)
fn scan_gbk_file(file_path: &Path) -> io::Result<Option<(usize, usize)>> {
    let mut file = fs::File::open(file_path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    if contains_chinese_utf8(&content) {
        // 如果是UTF-8编码且包含中文，直接返回None
        return Ok(None);
    }

    let total = count_chinese_gbk(&content);
    let streak = max_consecutive_gbk_chinese(&content);
    Ok(Some((total, streak)))
}

/// 将GBK内容转换为UTF-8并覆盖
fn convert_gbk_file(file_path: &Path, config: &Config) -> io::Result<()> {
    let mut file = fs::File::open(file_path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    match GBK.decode(&content, DecoderTrap::Strict) {
        Ok(decoded) => {
            if config.backup {
                let backup_path = file_path.with_extension(format!(
                    "{}.bak", file_path.extension().unwrap_or_default().to_string_lossy()
                ));
                fs::copy(file_path, &backup_path)?;
                if config.show_info {
                    println!("📦 已备份至：{}", backup_path.display());
                }
            }

            let mut file = fs::File::create(file_path)?;
            file.write_all(decoded.as_bytes())?;
            Ok(())
        }
        Err(_) => Err(io::Error::new(io::ErrorKind::InvalidData, "GBK 解码失败")),
    }
}


/// 判断并处理单个文件
fn handle_file(file_path: &Path, config: &Config) -> io::Result<()> {
    match scan_gbk_file(file_path)? {
        Some((count, streak)) => {
            if config.show_info || config.scan_only {
                println!("{}: 中文总数 = {}, 连续 = {}", file_path.display(), count, streak);
            }

            if count >= config.threshold && streak >= config.min_consecutive {
                if config.scan_only {
                    println!("🔍 可转换：{}", file_path.display());
                } else {
                    convert_gbk_file(file_path, config)?;
                    println!("✅ 已转换：{}", file_path.display());
                }
            }
        }
        None => {
            if config.show_info {
                println!("{} 是UTF-8编码或无中文，跳过", file_path.display());
            }
        }
    }

    Ok(())
}


/// 处理目录中所有文件
fn process_files_in_dir(
    dir: &Path,
    config: &Config,
    err: &mut HashMap<PathBuf, io::Error>,
) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();

        if path.is_dir() {
            process_files_in_dir(&path, config, err)?;
        } else if path.is_file() {
            let ext = path.extension().unwrap_or_default().to_string_lossy().to_lowercase();
            if config.extensions.iter().any(|e| e.to_lowercase() == ext) {
                if let Err(e) = handle_file(&path, config) {
                    err.insert(path.clone(), e);
                }
            }
        }
    }

    Ok(())
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

    let mut errors = HashMap::new();

    if let Err(e) = process_files_in_dir(Path::new(&config.dir), &config, &mut errors) {
        eprintln!("❌ 扫描目录失败: {}", e);
        return;
    }

    if !errors.is_empty() {
        println!("\n以下文件转换失败：");
        for (path, err) in &errors {
            println!("{}: {}", path.display(), err);
        }
    } else {
        println!("✅ 所有文件处理完成");
    }

    println!("\n按回车键退出...");
    let _ = io::stdin().read(&mut [0u8]).unwrap();
}
