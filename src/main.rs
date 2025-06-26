use chardetng::EncodingDetector;
use clap::Parser;
use encoding::all::GBK;
use encoding::{DecoderTrap, Encoding};
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::{env, fs};

/// GBK 转 UTF-8 工具（自动识别编码）
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Config {
    #[arg(short = 'd', long, default_value = "./", help = "要扫描的目录路径")]
    dir: String,

    #[arg(short = 'i', long = "show-info", help = "显示每个文件的编码猜测结果和置信度")]
    show_info: bool,

    #[arg(short = 's', long = "scan-only", help = "只扫描文件编码，不执行转换操作")]
    scan_only: bool,

    #[arg(short = 'b', long = "backup", help = "转换前将原文件备份为 .bak 文件")]
    backup: bool,

    #[arg(
        short = 'e',
        long = "extensions",
        value_delimiter = ',',
        default_value = "c,h",
        help = "要处理的文件扩展名（多个用英文逗号分隔）"
    )]
    extensions: Vec<String>,

    #[arg(
        short = 'm',
        long = "min-confidence",
        default_value_t = 0.8,
        help = "判断为 GBK 的最小置信度"
    )]
    min_confidence: f64,

    #[arg(long = "t", help = "指定顶级域名，如 cn、jp", default_value = "cn")]
    tld: Option<String>,
}

mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
/// 扫描 GBK 文件并返回编码和置信度
fn scan_gbk_file(file_path: &Path, config: &Config) -> io::Result<Option<(String, f64)>> {
    let mut file = fs::File::open(file_path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    if std::str::from_utf8(&content).is_ok() {
        return Ok(Some(("utf-8".to_string(), 1.0)));
    }

    let mut detector = EncodingDetector::new();
    detector.feed(&content, true);
    let tld_bytes = config.tld.as_deref().map(str::as_bytes);
    let (encoding, confident) = detector.guess_assess(tld_bytes, false);
    let name = encoding.name().to_lowercase();

    let confidence = if confident { 1.0 } else { 0.5 };

    if name == "gbk" && confidence >= config.min_confidence {
        Ok(Some((name, confidence)))
    } else if config.show_info {
        Ok(Some((name, confidence)))
    } else {
        Ok(None)
    }
}

/// 将 GBK 文件转换为 UTF-8
fn convert_gbk_file(file_path: &Path, config: &Config) -> io::Result<Option<PathBuf>> {
    let mut file = fs::File::open(file_path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    match GBK.decode(&content, DecoderTrap::Strict) {
        Ok(decoded) => {
            let mut backup_path = None;
            if config.backup {
                let bak = file_path.with_extension(format!(
                    "{}.bak",
                    file_path.extension().unwrap_or_default().to_string_lossy()
                ));
                fs::copy(file_path, &bak)?;
                backup_path = Some(bak);
            }

            let mut file = fs::File::create(file_path)?;
            file.write_all(decoded.as_bytes())?;
            Ok(backup_path)
        }
        Err(_) => Err(io::Error::new(io::ErrorKind::InvalidData, "GBK 解码失败")),
    }
}

/// 处理单个文件，根据配置进行扫描或转换
fn handle_file(file_path: &Path, config: &Config) -> io::Result<()> {
    match scan_gbk_file(file_path, config)? {
        Some((encoding_name, confidence)) => {
            let show_detail = |prefix: &str, msg: &str| {
                if config.show_info {
                    println!("{} {}: 编码 = {}, 置信度 = {:.2}{}", prefix, file_path.display(), encoding_name, confidence, msg);
                } else {
                    println!("{} {}: 编码 = {}{}", prefix, file_path.display(), encoding_name, msg);
                }
            };

            match encoding_name.as_str() {
                "utf-8" => {
                    show_detail("✅", "");
                }
                "gbk" => {
                    if config.scan_only {
                        show_detail("⏩", "，未转换（扫描模式）");
                    } else {
                        match convert_gbk_file(file_path, config) {
                            Ok(Some(bak)) if config.show_info => {
                                println!("📦 备份创建：{}", bak.display());
                            }
                            Ok(_) => {}
                            Err(e) => return Err(e),
                        }
                        show_detail("🔄", "，已转换为 UTF-8");
                    }
                }
                _ => {
                    show_detail("❌", "，跳过");
                }
            }
        }
        None => {
            println!("⚠️ {}: 编码不确定或置信度不足，跳过", file_path.display());
        }
    }

    Ok(())
}
/// 递归处理目录中的所有文件
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
            if config
                .extensions
                .iter()
                .any(|e| e.to_lowercase() == ext)
            {
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
