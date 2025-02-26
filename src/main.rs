use encoding::all::GBK;
use encoding::{DecoderTrap, Encoding};
use std::collections::HashMap;
use std::{env, fs};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

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

    // 统计中文字符数量
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


fn count_chinese_gbk(content: &[u8]) -> usize {
    let mut count = 0;

    for i in 0..content.len() - 1 {
        // 检查是否是 GB 2312 汉字区
        if content[i] >= 0xB0
            && content[i] <= 0xF7
            && content[i + 1] >= 0xA1
            && content[i + 1] <= 0xFE
        {
            count += 1;
        }
    }
    count
}

fn convert_gbk_to_utf8(file_path: &Path) -> io::Result<()> {
    let mut file = fs::File::open(file_path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    // 如果是有效的 UTF-8 编码且包含中文，跳过转换
    if contains_chinese_utf8(&content) {
        return Ok(());
    }

    // 检查文件中包含的中文字符数
    let chinese_count = count_chinese_gbk(&content);
    if chinese_count >= 4 {
        // 尝试将 GBK 编码的内容转换为 UTF-8
        match GBK.decode(&content, DecoderTrap::Strict) {
            Ok(decoded) => {
                // 直接替换原文件的内容
                let mut file = fs::File::create(file_path)?;
                file.write_all(decoded.as_bytes())?;
                println!(
                    "成功将 {} 转换为 UTF-8 格式并替换原文件",
                    file_path.display()
                );
                Ok(())
            }
            Err(_) => {
                println!("文件 {} 不是有效的 GBK 编码或转换失败", file_path.display());
                Err(io::Error::new(io::ErrorKind::InvalidData, "转换失败"))
            }
        }
    } else {
        Ok(())
    }
}

fn process_files_in_dir(dir: &Path, err: &mut HashMap<PathBuf, io::Error>) -> io::Result<()> {
    let paths = fs::read_dir(dir)?;

    for path in paths {
        let path = path?.path();

        if path.is_dir() {
            process_files_in_dir(&path, err)?;
        } else if path.is_file() {
            let extension = path.extension().unwrap_or_default();
            if extension.eq_ignore_ascii_case("c") || extension.eq_ignore_ascii_case("h") {
                // 只处理 .c 和 .h 文件
                if let Err(e) = convert_gbk_to_utf8(&path) {
                    err.insert(path.clone(), e);
                }
            }
        }
    }

    Ok(())
}

fn main() {
    // 打印版本号和编译信息
    println!(
        "This is version {}, built for {} by {}.[{}]\n",
        built_info::PKG_VERSION,
        built_info::TARGET,
        built_info::RUSTC_VERSION,
        built_info::BUILT_TIME_UTC
    );

    // 从命令行参数获取目录
    let dir = env::args().nth(1).unwrap_or_else(|| "./".to_string());
    
    let mut errors = HashMap::new();
    
    if let Err(e) = process_files_in_dir(Path::new(&dir), &mut errors) {
        eprintln!("扫描目录失败: {}", e);
        return;
    }
    
    if !errors.is_empty() {
        println!("以下文件转换失败：");
        for (path, err) in &errors {
            println!("文件 {} 转换失败: {}", path.display(), err);
        }
    } else {
        println!("所有文件转换成功");
    }
    println!("按回车键退出...");
    let _ = io::stdin().read(&mut [0u8]).unwrap();
}