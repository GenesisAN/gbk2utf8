use chardetng::EncodingDetector;
use clap::Parser;
use encoding::all::GBK;
use encoding::{DecoderTrap, Encoding};
use encoding_rs::Encoding as RsEncoding;
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::{env, fs};

/// GBK 转 UTF-8 工具（自动识别编码）
///
/// 本工具可递归扫描指定目录下的代码文件，检测是否为 GBK 编码，并可自动转换为 UTF-8。
/// 支持编码识别、条件过滤、文件备份等功能，适用于代码迁移或编码统一场景。
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Config {
    /// 要扫描的目录（默认当前目录）
    #[arg(
        short = 'd',
        long,
        default_value = "./",
        help = "要扫描的目录路径，默认为软件当前所在目录，支持递归"
    )]
    dir: String,

    /// 显示详细的编码检测信息
    #[arg(
        short = 'i',
        long = "show-info",
        help = "显示每个文件的编码猜测结果和置信度"
    )]
    show_info: bool,

    /// 仅扫描但不转换文件
    #[arg(
        short = 's',
        long = "scan-only",
        help = "只扫描文件编码，不执行转换操作"
    )]
    scan_only: bool,

    /// 在转换前备份原文件
    #[arg(short = 'b', long = "backup", help = "转换前将原文件备份为 .bak 文件")]
    backup: bool,

    /// 指定要扫描的文件扩展名（多个用逗号分隔）
    #[arg(
        short = 'e',
        long = "extensions",
        value_delimiter = ',',
        default_value = "c,h",
        help = "要处理的文件扩展名（多个用英文逗号分隔），例如：c,h,cpp"
    )]
    extensions: Vec<String>,

    /// 要求的最小置信度（用于更准确判断是否为中文 GBK 编码），高于此值才认为是 GBK 编码
    #[arg(
        short = 'm',
        long = "min-confidence",
        default_value_t = 0.8,
        help = "要求的最小置信度（用于更准确判断是否为中文 GBK 编码），高于此值才认为是 GBK 编码"
    )]
    min_confidence: f64,

    /// 指定顶级域名（用于提高 chardetng 的猜测准确性）
    #[arg(
        long = "t",
        help = "指定来源，如果要扫描相关语言的，可以尝试填写，用于提高猜测准确性(例如：cn、jp、kr..)",
        default_value = "cn"
    )]
    tld: Option<String>,
}

// 编译信息（若存在 build.rs 的话）
mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

/// 扫描文件是否为 GBK，并返回 (是否为GBK, 置信度标志)
fn scan_gbk_file(file_path: &Path, config: &Config) -> io::Result<Option<(String, f64)>> {
    let mut file = fs::File::open(file_path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    if std::str::from_utf8(&content).is_ok() {
        if config.show_info {
            println!("{}: 明确是 UTF-8，跳过", file_path.display());
        }
        // 返回特殊标识，告诉外部“别再打印了”
        return Ok(Some(("utf-8".to_string(), 1.0)));
    }

    let mut detector = EncodingDetector::new();
    detector.feed(&content, true);
    let tld_bytes = config.tld.as_deref().map(str::as_bytes);
    let (encoding, confident) = detector.guess_assess(tld_bytes, false);
    let name = encoding.name().to_lowercase();

    if name == "gbk" {
        let confidence = if confident { 1.0 } else { 0.5 };
        if confidence >= config.min_confidence {
            return Ok(Some((name, confidence)));
        } else {
            return Ok(None);
        }
    } else if config.show_info {
        let confidence = if confident { 1.0 } else { 0.5 };
        return Ok(Some((name, confidence)));
    } else {
        return Ok(None);
    }
}

/// 将 GBK 编码的文件转为 UTF-8
fn convert_gbk_file(file_path: &Path, config: &Config) -> io::Result<()> {
    let mut file = fs::File::open(file_path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    match GBK.decode(&content, DecoderTrap::Strict) {
        Ok(decoded) => {
            if config.backup {
                let backup_path = file_path.with_extension(format!(
                    "{}.bak",
                    file_path.extension().unwrap_or_default().to_string_lossy()
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
    match scan_gbk_file(file_path, config)? {
        Some((encoding_name, confidence)) => {
            if encoding_name == "utf-8" {
                // 已经在 scan_gbk_file 打印过，无需再次打印
                return Ok(());
            }

            if config.show_info {
                let extra_note = if encoding_name != "gbk" {
                    "，跳过"
                } else if config.scan_only {
                    "，可转换"
                } else {
                    "，已转换"
                };

                println!(
                    "{}: 猜测编码 = {}, 置信度 = {:.2}{}",
                    file_path.display(),
                    encoding_name,
                    confidence,
                    extra_note
                );
            }

            if encoding_name == "gbk" && !config.scan_only {
                convert_gbk_file(file_path, config)?;
            }
        }
        None => {
            if config.show_info {
                println!("{}: 无法确定为 GBK 或置信度不足，跳过", file_path.display());
            }
        }
    }
    Ok(())
}

/// 递归处理目录下所有目标文件
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
            let ext = path
                .extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase();
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
