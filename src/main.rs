use encoding::all::GBK;
use encoding::{DecoderTrap, Encoding};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

fn is_chinese(c: char) -> bool {
    // 判断字符是否在中文字符的 Unicode 范围内
    //参考：https://gist.github.com/shhider/38b43db6abe31b551384372254c5ac79
    //基本汉字	 20902字	4E00-9FA5
    //基本汉字补充	38字	9FA6-9FCB

    (c >= '\u{4e00}' && c <= '\u{9fa5}') || (c >= '\u{9fa6}' && c <= '\u{9fcb}')
}

fn contains_chinese_utf8(content: &[u8]) -> bool {
    let content_str = match std::str::from_utf8(content) {
        Ok(s) => s,
        Err(_) => return false, // 如果不是有效的 UTF-8 编码，则返回 false
    };

    content_str.chars().any(is_chinese)
}

fn contains_chinese_gbk(content: &[u8]) -> bool {
    // 判断汉字的GBK编码
    // https://zh.wikipedia.org/wiki/%E6%B1%89%E5%AD%97%E5%86%85%E7%A0%81%E6%89%A9%E5%B1%95%E8%A7%84%E8%8C%83#%E7%BC%96%E7%A0%81%E6%96%B9%E5%BC%8F
    // 范围         第1字节    第2字节            编码数    字数
    // 水准GBK/1    A1–A9      A1–FE              846      717
    // 水准GBK/2    B0–F7      A1–FE              6,768    6,763
    // 水准GBK/3    81–A0      40–FE (7F除外)     6,080    6,080
    // 水准GBK/4    AA–FE      40–A0 (7F除外)     8,160    8,160
    // 水准GBK/5    A8–A9      40–A0 (7F除外)     192      166
    // 用户定义     AA–AF      A1–FE              564
    // 用户定义     F8–FE      A1–FE              658
    // 用户定义     A1–A7      40–A0 (7F除外)     672
    // 检查是否包含 GB 2312 汉字区的字符（GBK 编码）、

    //这里只使用了GBK/2的范围
    for i in 0..content.len() - 1 {
        // 检查是否是 GB 2312 汉字区
        if content[i] >= 0xB0
            && content[i] <= 0xF7
            && content[i + 1] >= 0xA1
            && content[i + 1] <= 0xFE
        {
            return true;
        }
    }
    false
}

fn convert_gbk_to_utf8(file_path: &Path) -> io::Result<()> {
    // 读取文件内容
    let mut file = fs::File::open(file_path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    // 如果是有效的 UTF-8 编码且包含中文，跳过转换
    if contains_chinese_utf8(&content) {
        // println!("文件 {} 是 UTF-8 编码且包含中文，跳过转换", file_path.display());
        return Ok(());
    }

    // 判断是否包含 GBK 编码的中文字符
    if contains_chinese_gbk(&content) {
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
        //println!("文件 {} 不包含中文 GBK 字符，跳过转换", file_path.display());
        Ok(())
    }
}

fn process_files_in_dir(dir: &Path) -> io::Result<()> {
    // 递归地扫描目录下的所有文件
    let paths = fs::read_dir(dir)?;

    for path in paths {
        let path = path?.path();

        if path.is_dir() {
            // 如果是目录，递归调用
            process_files_in_dir(&path)?;
        } else if path.is_file() {
            let extension = path.extension().unwrap_or_default();
            if extension == "c" || extension == "h" {
                // 只处理 .c 和 .h 文件
                if let Err(e) = convert_gbk_to_utf8(&path) {
                    eprintln!("处理文件 {} 时出错: {}", path.display(), e);
                }
            }
        }
    }

    Ok(())
}

fn main() {
    let dir = "./"; // 设置要扫描的目录，默认为当前目录

    if let Err(e) = process_files_in_dir(Path::new(dir)) {
        eprintln!("处理文件夹时出错: {}", e);
    }
}
