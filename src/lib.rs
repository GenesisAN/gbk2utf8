use chardetng::EncodingDetector;
use clap::Parser;
use encoding::all::GBK;
use encoding::{DecoderTrap, Encoding};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

/// GBK 转 UTF-8 工具（自动识别编码）
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Config {
    #[arg(short = 'd', long, default_value = "./", help = "要扫描的目录路径")]
    pub dir: String,

    #[arg(short = 'i', long = "show-info", help = "显示每个文件的编码猜测结果和置信度")]
    pub show_info: bool,

    #[arg(short = 's', long = "scan-only", help = "只扫描文件编码，不执行转换操作")]
    pub scan_only: bool,

    #[arg(short = 'b', long = "backup", help = "转换前将原文件备份为 .bak 文件")]
    pub backup: bool,

    #[arg(
        short = 'e',
        long = "extensions",
        value_delimiter = ',',
        default_value = "c,h,txt",
        help = "要处理的文件扩展名（多个用英文逗号分隔）"
    )]
    pub extensions: Vec<String>,

    #[arg(
        short = 'm',
        long = "min-confidence",
        default_value_t = 0.8,
        help = "判断为 GBK 的最小置信度"
    )]
    pub min_confidence: f64,

    #[arg(long = "t", help = "指定顶级域名，如 cn、jp", default_value = "cn")]
    pub tld: Option<String>,

    #[arg(
        long = "ignore-file",
        default_value = ".gbk2utf8ignore",
        help = "忽略规则文件路径（gitignore 语法），相对路径基于 --dir"
    )]
    pub ignore_file: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileProcessOutcome {
    Converted,
    NoConversion,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ProcessingStats {
    pub converted: usize,
    pub failed: usize,
    pub no_conversion: usize,
}

#[derive(Debug, Default)]
pub struct RunResult {
    pub errors: HashMap<PathBuf, io::Error>,
    pub stats: ProcessingStats,
}

/// 扫描 GBK 文件并返回编码和置信度
pub fn scan_gbk_file(file_path: &Path, config: &Config) -> io::Result<Option<(String, f64)>> {
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
pub fn convert_gbk_file(file_path: &Path, config: &Config) -> io::Result<Option<PathBuf>> {
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
pub fn handle_file(file_path: &Path, config: &Config) -> io::Result<FileProcessOutcome> {
    match scan_gbk_file(file_path, config)? {
        Some((encoding_name, confidence)) => {
            let show_detail = |prefix: &str, msg: &str| {
                if config.show_info {
                    println!(
                        "{} {}: 编码 = {}, 置信度 = {:.2}{}",
                        prefix,
                        file_path.display(),
                        encoding_name,
                        confidence,
                        msg
                    );
                } else {
                    println!("{} {}: 编码 = {}{}", prefix, file_path.display(), encoding_name, msg);
                }
            };

            match encoding_name.as_str() {
                "utf-8" => {
                    show_detail("✅", "");
                    return Ok(FileProcessOutcome::NoConversion);
                }
                "gbk" => {
                    if config.scan_only {
                        show_detail("⏩", "，未转换（扫描模式）");
                        return Ok(FileProcessOutcome::NoConversion);
                    } else {
                        match convert_gbk_file(file_path, config) {
                            Ok(Some(bak)) if config.show_info => {
                                println!("📦 备份创建：{}", bak.display());
                            }
                            Ok(_) => {}
                            Err(e) => return Err(e),
                        }
                        show_detail("🔄", "，已转换为 UTF-8");
                        return Ok(FileProcessOutcome::Converted);
                    }
                }
                _ => {
                    show_detail("❌", "，跳过");
                    return Ok(FileProcessOutcome::NoConversion);
                }
            }
        }
        None => {
            println!("⚠️ {}: 编码不确定或置信度不足，跳过", file_path.display());
            return Ok(FileProcessOutcome::NoConversion);
        }
    }
}

pub fn build_ignore_matcher(root_dir: &Path, config: &Config) -> io::Result<Gitignore> {
    let mut builder = GitignoreBuilder::new(root_dir);
    let absolute_ignore_file = resolve_ignore_file_path(root_dir, config);

    if absolute_ignore_file.exists() {
        if let Some(e) = builder.add(&absolute_ignore_file) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, e.to_string()));
        }
        println!("🚫 忽略规则文件：{}", absolute_ignore_file.display());
    }

    builder
        .build()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))
}

fn resolve_ignore_file_path(root_dir: &Path, config: &Config) -> PathBuf {
    let ignore_file_path = Path::new(&config.ignore_file);
    if ignore_file_path.is_absolute() {
        ignore_file_path.to_path_buf()
    } else {
        root_dir.join(ignore_file_path)
    }
}

pub fn should_ignore(path: &Path, is_dir: bool, ignore_matcher: &Gitignore) -> bool {
    ignore_matcher.matched(path, is_dir).is_ignore()
}

/// 递归处理目录中的所有文件
pub fn process_files_in_dir(
    root_dir: &Path,
    dir: &Path,
    config: &Config,
    ignore_matcher: &Gitignore,
    err: &mut HashMap<PathBuf, io::Error>,
    stats: &mut ProcessingStats,
) -> io::Result<()> {
    let ignore_file_path = resolve_ignore_file_path(root_dir, config);

    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        let relative_path = path.strip_prefix(root_dir).unwrap_or(&path);

        if path.is_file() && path == ignore_file_path {
            continue;
        }

        if should_ignore(relative_path, path.is_dir(), ignore_matcher) {
            if config.show_info {
                println!("🚫 {}: 命中忽略规则，跳过", path.display());
            }
            continue;
        }

        if path.is_dir() {
            process_files_in_dir(root_dir, &path, config, ignore_matcher, err, stats)?;
        } else if path.is_file() {
            let ext = path
                .extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase();
            if config.extensions.iter().any(|e| e.to_lowercase() == ext) {
                match handle_file(&path, config) {
                    Ok(FileProcessOutcome::Converted) => stats.converted += 1,
                    Ok(FileProcessOutcome::NoConversion) => stats.no_conversion += 1,
                    Err(e) => {
                        stats.failed += 1;
                        err.insert(path.clone(), e);
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn run(config: &Config) -> io::Result<RunResult> {
    let root_dir = PathBuf::from(&config.dir);
    let ignore_matcher = build_ignore_matcher(&root_dir, config)?;
    let mut errors = HashMap::new();
    let mut stats = ProcessingStats::default();

    process_files_in_dir(
        &root_dir,
        &root_dir,
        config,
        &ignore_matcher,
        &mut errors,
        &mut stats,
    )?;
    Ok(RunResult { errors, stats })
}
