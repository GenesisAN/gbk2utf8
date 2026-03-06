use encoding::all::GBK;
use encoding::{EncoderTrap, Encoding};
use gbk2utf8::{
    build_ignore_matcher, convert_gbk_file, handle_file, process_files_in_dir, run,
    scan_gbk_file, should_ignore, Config, FileProcessOutcome, ProcessingStats,
};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tempfile::{tempdir, TempDir};

fn make_config(dir: &Path) -> Config {
    Config {
        dir: dir.to_string_lossy().to_string(),
        show_info: false,
        scan_only: false,
        backup: false,
        extensions: vec!["c".to_string(), "h".to_string(), "txt".to_string()],
        min_confidence: 0.8,
        tld: Some("cn".to_string()),
        ignore_file: ".gbk2utf8ignore".to_string(),
    }
}

fn gbk_bytes(content: &str) -> Vec<u8> {
    GBK.encode(content, EncoderTrap::Strict)
        .expect("encode test text to gbk")
}

struct TestProject {
    temp_dir: TempDir,
}

impl TestProject {
    fn new() -> Self {
        Self {
            temp_dir: tempdir().expect("create temp dir"),
        }
    }

    fn root(&self) -> &Path {
        self.temp_dir.path()
    }

    fn path(&self, rel: &str) -> PathBuf {
        self.root().join(rel)
    }

    fn write_utf8(&self, rel: &str, content: &str) -> PathBuf {
        let p = self.path(rel);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).expect("create parent dir");
        }
        fs::write(&p, content).expect("write utf8 file");
        p
    }

    fn write_gbk(&self, rel: &str, content: &str) -> PathBuf {
        let p = self.path(rel);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).expect("create parent dir");
        }
        fs::write(&p, gbk_bytes(content)).expect("write gbk file");
        p
    }

    fn write_bytes(&self, rel: &str, bytes: &[u8]) -> PathBuf {
        let p = self.path(rel);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).expect("create parent dir");
        }
        fs::write(&p, bytes).expect("write binary file");
        p
    }

    fn write_ignore(&self, content: &str) {
        self.write_utf8(".gbk2utf8ignore", content);
    }
}

// 扫描UTF-8 文件应该返回 UTF-8 编码和置信度 1.0
#[test]
fn scan_utf8_file_returns_utf8() {
    let project = TestProject::new();
    let file = project.write_utf8("utf8.c", "hello utf8");

    let config = make_config(project.root());
    let scanned = scan_gbk_file(&file, &config).expect("scan file");

    assert_eq!(scanned, Some(("utf-8".to_string(), 1.0)));
}

// 扫描 GBK 文件应该返回 GBK 编码和置信度
#[test]
fn scan_gbk_file_respects_min_confidence() {
    let project = TestProject::new();
    let input = "中文内容用于编码识别，包含足够多的汉字来提高检测准确度。中文内容用于编码识别。";
    let file = project.write_gbk("legacy.c", input);

    let mut config = make_config(project.root());
    config.min_confidence = 0.5;
    let scanned = scan_gbk_file(&file, &config).expect("scan gbk file");
    assert!(matches!(scanned, Some((ref name, _)) if name == "gbk"));

    config.min_confidence = 1.1;
    let filtered = scan_gbk_file(&file, &config).expect("scan gbk file with high threshold");
    assert!(filtered.is_none());
}

#[test]
fn convert_gbk_file_creates_backup_and_converts() {
    let project = TestProject::new();
    let input = "测试转换内容";
    let file = project.write_gbk("legacy.h", input);

    let mut config = make_config(project.root());
    config.backup = true;

    let backup = convert_gbk_file(&file, &config)
        .expect("convert gbk file")
        .expect("backup should exist");

    let converted = fs::read_to_string(&file).expect("read converted utf8 file");
    let backup_bytes = fs::read(&backup).expect("read backup file");

    assert_eq!(converted, input);
    assert_eq!(backup_bytes, gbk_bytes(input));
}

#[test]
fn convert_gbk_file_returns_error_for_invalid_data() {
    let project = TestProject::new();
    let file = project.write_bytes("invalid.c", &[0xFF, 0xFF, 0xFF]);

    let config = make_config(project.root());
    let err = convert_gbk_file(&file, &config).expect_err("invalid gbk should fail");

    assert_eq!(err.kind(), io::ErrorKind::InvalidData);
}

#[test]
fn handle_file_scan_only_keeps_original_bytes() {
    let project = TestProject::new();
    let file = project.path("scan_only.c");
    let original = gbk_bytes("仅扫描不转换");
    project.write_bytes("scan_only.c", &original);

    let mut config = make_config(project.root());
    config.scan_only = true;

    let outcome = handle_file(&file, &config).expect("handle file in scan only mode");
    assert_eq!(outcome, FileProcessOutcome::NoConversion);

    let after = fs::read(&file).expect("read file after scan only");
    assert_eq!(after, original);
}

#[test]
fn ignore_matcher_uses_ignore_file_rules() {
    let project = TestProject::new();
    project.write_ignore("skip/\n*.bak\n");

    let config = make_config(project.root());
    let matcher = build_ignore_matcher(project.root(), &config).expect("build ignore matcher");

    assert!(should_ignore(Path::new("skip"), true, &matcher));
    assert!(should_ignore(Path::new("note.c.bak"), false, &matcher));
    assert!(!should_ignore(Path::new("keep/main.c"), false, &matcher));
}

#[test]
fn process_files_in_dir_applies_ignore_and_extension_filters() {
    let project = TestProject::new();
    let keep_c = project.write_gbk("keep/main.c", "需要被转换的c文件");
    let keep_txt = project.write_gbk("keep/note.txt", "需要被转换的txt文件");
    let keep_rs = project.write_gbk("keep/lib.rs", "扩展名不匹配，不应转换");
    let skip_c = project.write_gbk("skip/legacy.c", "命中忽略规则，不应转换");
    let keep_rs_bytes = fs::read(&keep_rs).expect("read rs bytes");
    let skip_c_bytes = fs::read(&skip_c).expect("read skipped c bytes");
    project.write_ignore("skip/\n");

    let mut config = make_config(project.root());
    config.extensions = vec!["c".to_string(), "txt".to_string()];

    let matcher = build_ignore_matcher(project.root(), &config).expect("build ignore matcher");
    let mut errors = HashMap::new();
    let mut stats = ProcessingStats::default();

    process_files_in_dir(
        project.root(),
        project.root(),
        &config,
        &matcher,
        &mut errors,
        &mut stats,
    )
    .expect("process files in dir");

    assert!(errors.is_empty());
    assert_eq!(stats.converted, 2);
    assert_eq!(stats.failed, 0);
    assert_eq!(stats.no_conversion, 0);
    assert_eq!(
        fs::read_to_string(&keep_c).expect("read converted c"),
        "需要被转换的c文件"
    );
    assert_eq!(
        fs::read_to_string(&keep_txt).expect("read converted txt"),
        "需要被转换的txt文件"
    );
    assert_eq!(fs::read(&keep_rs).expect("read rs file"), keep_rs_bytes);
    assert_eq!(fs::read(&skip_c).expect("read skipped c file"), skip_c_bytes);
}

#[test]
fn run_end_to_end_with_generated_files() {
    let project = TestProject::new();
    let converted = project.write_gbk("src/main.c", "端到端转换文件");
    let ignored = project.write_gbk("target/legacy.c", "应被忽略");
    let untouched = project.write_gbk("src/readme.md", "扩展名不匹配");
    let ignored_before = fs::read(&ignored).expect("read ignored before");
    let untouched_before = fs::read(&untouched).expect("read untouched before");

    project.write_ignore("target/\n");

    let mut config = make_config(project.root());
    config.extensions = vec!["c".to_string()];

    let result = run(&config).expect("run end-to-end should succeed");
    assert!(result.errors.is_empty());
    assert_eq!(result.stats.converted, 1);
    assert_eq!(result.stats.failed, 0);
    assert_eq!(result.stats.no_conversion, 0);

    assert_eq!(
        fs::read_to_string(&converted).expect("read converted file"),
        "端到端转换文件"
    );
    assert_eq!(fs::read(&ignored).expect("read ignored file"), ignored_before);
    assert_eq!(fs::read(&untouched).expect("read untouched file"), untouched_before);
}
