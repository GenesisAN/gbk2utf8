# GBK 转 UTF-8 工具（自动识别编码）
由于大部分转码工具都是直接批量转换，可能会误将 UTF-8 作为 GBK 再转换为 UTF-8，导致乱码。
因此我编写了这个工具，**先识别文件编码，再决定是否转换**，从而避免误操作。

UTF-8 编码识别基于 Rust 标准库的 `std::str::from_utf8`，如果文件确认是 UTF-8 则直接忽略不进行处理，GBK 编码识别使用了 `chardetng` 库。高于 0.8 的置信度才会被认为是 GBK 编码，然后进行转换。

---

## ⚡ 快速开始

默认情况下，工具会处理 `.c`、`.h`、`.txt` 三类文件。
如果你想只转换 `.txt` 文件，推荐直接执行：

```bash
gbk2utf8 -e txt -b
```

说明：

* `-e txt` 表示仅处理 `.txt` 文件
* `-b` 会先备份原文件为 `.bak`，更安全

---

## 🛠 安装与升级（推荐）

这个项目已经可以作为标准 Rust 命令行工具使用，你可以通过 `cargo install` 安装到本机全局命令目录，不再需要手动复制可执行文件。

说明：安装后可执行文件会放在 `~/.cargo/bin`（Windows 通常是 `%USERPROFILE%\\.cargo\\bin`），请确保该目录已加入 `PATH`。

### 方式 1：当前仓库本地安装

```bash
cargo install --path .
```

升级（覆盖安装最新本地代码）：

```bash
cargo install --path . --force
```

### 方式 2：从 Git 仓库安装

```bash
cargo install --git <你的仓库地址> gbk2utf8
```

升级：

```bash
cargo install --git <你的仓库地址> gbk2utf8 --force
```

### 方式 3 通过 crates.io 安装

```bash
cargo install gbk2utf8
```

升级：

```bash
cargo install gbk2utf8 --force
```

安装完成后，直接在任意目录执行：

```bash
gbk2utf8 --help
gbk2utf8 -d . -e txt -b
```

---

## 🚀 使用示例

### 转换当前目录下默认扩展名文件（`.txt`、`.c`、`.h`）

```bash
gbk2utf8
```
⚠️ 警告：❗转换操作具有破坏性，建议始终备份原文件。

备份功能 默认关闭，如果需要备份，请使用 `-b` 或 `--backup` 参数。
### 扫描 `src/` 目录，显示编码信息但不转换：

```bash
gbk2utf8 -d ./src -i -s
```

### 转换 `.txt` 文件，启用备份：

```bash
gbk2utf8 -e "txt" -b
```

### 仅转换代码文件（`.c`、`.h`）：

```bash
gbk2utf8 -e "c,h"
```

### 使用忽略规则文件（类似 `.gitignore`）

默认会读取扫描根目录下的 `.gbk2utf8ignore`，文件存在时自动生效；也可以手动指定：

```bash
gbk2utf8 -d ./src --ignore-file .gbk2utf8ignore
```

`.gbk2utf8ignore` 示例：

```text
# 忽略整个目录
build/
target/

# 忽略特定文件
legacy/old.c

# 忽略所有备份文件
*.bak
```

---

## 🔧 命令行参数说明

| 参数                           | 说明                                 |
| ---------------------------- | ---------------------------------- |
| `-d, --dir <路径>`             | 要扫描的目录，默认是当前目录，支持递归子目录             |
| `-e, --extensions <扩展名,...>` | 只处理指定扩展名的文件（默认：`txt,c,h`），多个用英文逗号分隔    |
| `-s, --scan-only`            | 仅扫描，不执行转换                          |
| `-b, --backup`               | 转换前为每个文件备份为 `.bak`                 |
| `-i, --show-info`            | 显示每个文件的编码检测信息与置信度                  |
| `-m, --min-confidence <数值>`  | 置信度阈值（0～1），低于该值不会视为 GBK，默认 0.8     |
| `--t <TLD>`                  | 顶级域名（如 `cn`、`jp`）用于提高识别准确度，默认 `cn` |
| `--ignore-file <路径>`         | 忽略规则文件（gitignore 语法），默认 `.gbk2utf8ignore` |

---

## ✨ 功能特性

* ✅ 自动识别文件编码（支持 UTF-8 / GBK 等）
* ✅ 识别 UTF-8 文件，避免误转码
* ✅ 可递归扫描子目录
* ✅ 支持 gitignore 语法的忽略规则（可跳过指定目录/文件）
* ✅ 支持指定要转换和识别的文件扩展名
* ✅ 支持转换前备份 `.bak`
* ✅ 可显示每个文件的编码猜测和置信度
* ✅ 结束时输出统计信息（成功转换 / 转换失败 / 无需转换）

---

## 📊 结束统计说明

程序结束时会输出三类统计：

1. `成功转换`：识别为 GBK 且已完成 UTF-8 转换的文件数量。
2. `转换失败`：进入处理流程但转换失败的文件数量。
3. `无需转换`：进入处理流程但不需要写回的文件数量（如已是 UTF-8、扫描模式下不执行转换、编码不满足转换条件）。

注意：命中忽略规则（如 `.gbk2utf8ignore`）的文件不会计入以上任何统计项。

---

## 🧠 原理说明

### 📍 文件编码识别逻辑

工具使用 [`chardetng`](https://github.com/hsivonen/chardetng) 识别编码，流程如下：

1. **尝试将文件内容解析为 UTF-8**：

   * 如果成功，直接认定是 UTF-8，无需转换。
   * 这样可以防止将本就是 UTF-8 的文件误判成 GBK 并错误转码。

2. **使用 chardetng 猜测编码**：

   * 基于字节模式、语言特征（如 `--t cn`）进行启发式分析。
   * 如果识别为 `GBK` 且置信度超过设定值（如 `0.8`），则判定为 GBK。

### 📦 备份逻辑

如果开启 `--backup`，转换前会自动复制原文件为 `.bak` 文件，保留原始内容。

例如：

```text
main.c -> main.c.bak
```

---

## 🧪 支持识别的编码

虽然工具只转换 GBK，但能识别包括：

* UTF-8
* GBK / GB2312 / GB18030（作为 GBK 的超集）
* Shift-JIS、EUC-KR 等（被排除）

---

## 💬 示例输出（含 `--show-info`）

```text
./src/main.c: 明确是 UTF-8，跳过
./lib/legacy.c: 猜测编码 = gbk, 置信度 = 1.00，已转换
./old/driver.c: 猜测编码 = windows-1252, 置信度 = 0.50，跳过
```

---

## 📦 编译与发布

### 使用 `cargo` 构建

```bash
git clone https://github.com/GenesisAN/gbk2utf8.git
cd gbk2utf8
cargo build --release
./target/release/gbk2utf8 --help
```

### 自动发布（GitHub Release）

仓库已内置自动发布工作流：`.github/workflows/release.yml`

触发方式：

1. 推送版本标签（推荐）：`v*`，例如 `v0.1.3`
2. 手动触发 workflow（`workflow_dispatch`）并填写 `tag`

发布内容：

* 自动构建 Linux / Windows / macOS 的 release 二进制
* 自动创建 GitHub Release 并上传构建产物

产物命名示例：

* `gbk2utf8-linux-x86_64`
* `gbk2utf8-windows-x86_64.exe`
* `gbk2utf8-macos-x86_64`

