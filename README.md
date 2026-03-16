# gbk2utf8

[中文](#中文) | [English](#english)

---

## 中文

### GBK 转 UTF-8 工具（自动识别编码）

由于大部分转码工具是直接批量转换，可能会误将 UTF-8 当作 GBK 再转为 UTF-8，导致乱码。
本工具会先识别文件编码，再决定是否转换，从而减少误操作。

识别策略：
- UTF-8 识别基于 Rust 标准库 `std::str::from_utf8`
- 非 UTF-8 使用 `chardetng` 进行编码猜测
- 仅当识别为 GBK 且置信度达到阈值（默认 0.8）时才执行转换

---

### ⚡ 快速开始（默认推荐）

先通过 Cargo 安装：

```bash
cargo install gbk2utf8
```

验证安装：

```bash
gbk2utf8 --help
```

强制英文输出：

```bash
gbk2utf8 --lang en --help
```

默认会处理 `.txt`、`.c`、`.h` 文件。仅处理 txt 并自动备份：

```bash
gbk2utf8 -e txt -b
```

---

### 🛠 安装与升级

安装后可执行文件位于 `~/.cargo/bin`（Windows 通常是 `%USERPROFILE%\\.cargo\\bin`），请确保该目录在 `PATH` 中。

推荐按下面 3 条路径选择一种：

1. 通过 Cargo 安装（推荐，大多数用户）

如果你已安装 Cargo，直接执行：

```bash
cargo install gbk2utf8
```

升级：

```bash
cargo install gbk2utf8 --force
```

如果你还没安装 Cargo（不做 Rust 开发也可以安装）：

1. 访问 `https://rustup.rs` 并安装 Rustup（会同时安装 Cargo）
2. 重新打开终端，确认 Cargo 可用：

```bash
cargo --version
```

2. 直接使用预编译 EXE（不安装 Cargo）

1. 打开 GitHub Releases：`https://github.com/GenesisAN/gbk2utf8/releases`
2. 下载 Windows 产物（如 `gbk2utf8-windows-x86_64.exe`）
3. 重命名为 `gbk2utf8.exe`（可选）
4. 放到任意目录并执行，或把该目录加入 `PATH` 后全局调用

3. 开发者安装方式（本地源码 / Git）

本地源码安装：

```bash
cargo install --path .
cargo install --path . --force
```

从 Git 仓库安装：

```bash
cargo install --git https://github.com/GenesisAN/gbk2utf8.git gbk2utf8
cargo install --git https://github.com/GenesisAN/gbk2utf8.git gbk2utf8 --force
```

卸载：

```bash
cargo uninstall gbk2utf8
```

---

### 🚀 使用示例

处理当前目录下默认扩展名（`.txt`、`.c`、`.h`）：

```bash
gbk2utf8
```

扫描但不转换：

```bash
gbk2utf8 -d ./src -i -s
```

仅处理 txt 并备份：

```bash
gbk2utf8 -e txt -b
```

仅处理代码文件：

```bash
gbk2utf8 -e c,h
```

使用忽略规则文件：

```bash
gbk2utf8 -d ./src --ignore-file .gbk2utf8ignore
```

`.gbk2utf8ignore` 示例：

```text
build/
target/
legacy/old.c
*.bak
```

---

### 🔧 命令行参数

| 参数 | 说明 |
| --- | --- |
| `-d, --dir <路径>` | 扫描目录（默认当前目录），递归处理子目录 |
| `-e, --extensions <扩展名,...>` | 处理的扩展名，默认 `txt,c,h` |
| `-s, --scan-only` | 仅扫描，不转换 |
| `-b, --backup` | 转换前备份为 `.bak` |
| `-i, --show-info` | 显示编码猜测与置信度 |
| `-m, --min-confidence <数值>` | GBK 置信度阈值，默认 `0.8` |
| `--t <TLD>` | 顶级域名提示（如 `cn`、`jp`），默认 `cn` |
| `--ignore-file <路径>` | 忽略规则文件（gitignore 语法），默认 `.gbk2utf8ignore` |
| `--lang <auto\|zh\|en>` | 输出语言，默认 `auto`（自动检测） |

---

### ✨ 功能特性

- 自动识别编码（UTF-8 / GBK）
- 避免误转 UTF-8 文件
- 递归目录扫描
- 支持 gitignore 风格忽略规则
- 支持扩展名过滤
- 支持转换前备份
- 支持显示编码检测详情
- 输出转换统计信息

---

### 📊 统计口径

程序结束会输出：
1. 成功转换
2. 转换失败
3. 无需转换

命中忽略规则的文件不计入以上统计。

---

### 📦 构建与发布

本地构建：

```bash
git clone https://github.com/GenesisAN/gbk2utf8.git
cd gbk2utf8
cargo build --release
./target/release/gbk2utf8 --help
```

仓库内置 GitHub Release 工作流：`.github/workflows/release.yml`。

---

## English

### GBK to UTF-8 CLI (encoding-aware conversion)

Many batch converters convert files blindly and may corrupt already UTF-8 files.
`gbk2utf8` detects encoding first, then converts only when appropriate.

Detection strategy:
- UTF-8 check via Rust stdlib `std::str::from_utf8`
- Non-UTF-8 detection via `chardetng`
- Convert only when encoding is GBK and confidence is above threshold (default `0.8`)

---

### ⚡ Quick Start (recommended)

Install from crates.io:

```bash
cargo install gbk2utf8
```

Verify:

```bash
gbk2utf8 --help
```

Force English output:

```bash
gbk2utf8 --lang en --help
```

Default file extensions are `.txt`, `.c`, `.h`.
Convert only txt files with backup:

```bash
gbk2utf8 -e txt -b
```

---

### 🛠 Install and Upgrade

Binary location is usually `~/.cargo/bin` (Windows: `%USERPROFILE%\\.cargo\\bin`).
Make sure it is in your `PATH`.

Choose one of these 3 paths:

1. Install with Cargo (recommended for most users)

If Cargo is already installed:

```bash
cargo install gbk2utf8
```

Upgrade:

```bash
cargo install gbk2utf8 --force
```

If Cargo is not installed yet (no Rust development needed):

1. Install Rustup from `https://rustup.rs` (it also installs Cargo)
2. Reopen your terminal and verify:

```bash
cargo --version
```

2. Use prebuilt executable (without Cargo)

1. Open GitHub Releases: `https://github.com/GenesisAN/gbk2utf8/releases`
2. Download the Windows artifact (for example `gbk2utf8-windows-x86_64.exe`)
3. Optionally rename it to `gbk2utf8.exe`
4. Run it directly, or add its folder to `PATH` for global usage

3. Developer install (local source / Git)

Install from local source:

```bash
cargo install --path .
cargo install --path . --force
```

Install from Git repository:

```bash
cargo install --git https://github.com/GenesisAN/gbk2utf8.git gbk2utf8
cargo install --git https://github.com/GenesisAN/gbk2utf8.git gbk2utf8 --force
```

Uninstall:

```bash
cargo uninstall gbk2utf8
```

---

### 🚀 Usage Examples

Process default extensions in current directory:

```bash
gbk2utf8
```

Scan only (no conversion):

```bash
gbk2utf8 -d ./src -i -s
```

Only txt with backup:

```bash
gbk2utf8 -e txt -b
```

Only C headers/sources:

```bash
gbk2utf8 -e c,h
```

Use ignore rules:

```bash
gbk2utf8 -d ./src --ignore-file .gbk2utf8ignore
```

---

### 🔧 CLI Options

| Option | Description |
| --- | --- |
| `-d, --dir <DIR>` | Directory to scan recursively (default: current directory) |
| `-e, --extensions <EXTENSIONS,...>` | File extensions to process (default: `txt,c,h`) |
| `-s, --scan-only` | Scan only, do not convert |
| `-b, --backup` | Create `.bak` before conversion |
| `-i, --show-info` | Show detected encoding and confidence |
| `-m, --min-confidence <VALUE>` | GBK confidence threshold (default: `0.8`) |
| `--t <TLD>` | TLD hint like `cn`, `jp` (default: `cn`) |
| `--ignore-file <PATH>` | Ignore rules file in gitignore syntax (default: `.gbk2utf8ignore`) |
| `--lang <auto\|zh\|en>` | Output language (default: `auto`, auto-detected) |

---

### ✨ Features

- Encoding-aware conversion (UTF-8 / GBK)
- Avoids accidental conversion of UTF-8 files
- Recursive directory traversal
- gitignore-style ignore rules
- Extension filtering
- Optional backup before write
- Per-file detection details
- Final conversion statistics

---

### 📦 Build and Release

Build locally:

```bash
git clone https://github.com/GenesisAN/gbk2utf8.git
cd gbk2utf8
cargo build --release
./target/release/gbk2utf8 --help
```

GitHub Release workflow is available at `.github/workflows/release.yml`.
