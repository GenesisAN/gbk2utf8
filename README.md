# GBK2UTF8 转换工具

该工具用于批量将 `.c` 和 `.h` 文件中的 GBK 编码中文字符转换为 UTF-8 编码，并直接替换原文件。它能够递归地扫描指定目录及其子目录中的 `.c` 和 `.h` 文件，判断文件中是否包含中文字符，并将 GBK 编码的中文转换为 UTF-8 格式。

## 功能

- 递归遍历指定目录及其子目录中的 `.c` 和 `.h` 文件。
- 识别并转换 GBK 编码的中文字符为 UTF-8 格式。
- 如果文件已是 UTF-8 编码且包含中文字符，则跳过该文件。
- 直接替换原始文件，不会生成新文件。

## 依赖

此工具使用了 Rust 编程语言的 `encoding` 库，主要用于处理字符编码转换。

- `encoding`：用于支持 GBK 编码和 UTF-8 编码之间的转换。

## 安装

1. 确保你已安装 Rust 环境。可以通过 [Rust 官方网站](https://www.rust-lang.org/)下载并安装 Rust。
2. 下载并克隆本项目

3. 使用 `cargo` 构建项目：

   ```bash
   cargo build --release
   ```

## 使用方法

1. 进入项目目录：

   ```bash
   cd gbk2utf8
   ```

2. 运行程序，指定要扫描的目录，默认为当前目录：

   ```bash
   cargo run
   ```

3. 程序会扫描指定目录中的所有 `.c` 和 `.h` 文件，检查它们是否包含 GBK 编码的中文字符。如果文件是 GBK 编码并且包含中文字符，它将转换为 UTF-8 编码，并直接替换原文件。

### 示例

假设你有一个包含多个 `.c` 和 `.h` 文件的项目，执行以下命令：

```bash
cargo run
```

程序会遍历当前目录及其所有子目录中的 `.c` 和 `.h` 文件，将其中的 GBK 编码中文转换为 UTF-8 编码，并直接替换原文件。如果文件已经是 UTF-8 编码或不包含中文，程序会跳过该文件。

## 注意事项

- 该程序会直接修改原始文件，因此请确保在运行程序之前做好备份。
- 本程序仅支持 `.c` 和 `.h` 文件，其他类型的文件将被忽略。
- 如果文件无法从 GBK 解码，程序会跳过该文件并输出错误信息。


