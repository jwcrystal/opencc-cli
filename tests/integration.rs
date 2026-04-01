use std::fs;
use std::path::PathBuf;
use std::process::Command;

struct TestContext {
    tmpdir: PathBuf,
}

impl TestContext {
    fn new() -> Self {
        let tmpdir = tempfile::tempdir().unwrap().keep();
        Self { tmpdir }
    }

    fn run(&self, args: &[&str]) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_opencc-cli"));
        cmd.args(args);
        cmd
    }

    fn create_file(&self, name: &str, content: &str) -> PathBuf {
        let path = self.tmpdir.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, content).unwrap();
        path
    }

    fn read_file(&self, name: &str) -> String {
        fs::read_to_string(self.tmpdir.join(name)).unwrap()
    }
}

// --- Text mode ---

#[test]
fn text_s2t() {
    let ctx = TestContext::new();
    let output = ctx
        .run(&["-m", "s2t", "-t", "开放中文转换"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("開放中文轉換"));
}

#[test]
fn text_t2s() {
    let ctx = TestContext::new();
    let output = ctx
        .run(&["-m", "t2s", "-t", "開放中文轉換"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("开放中文转换"));
}

#[test]
fn text_s2twp_taiwan_vocab() {
    let ctx = TestContext::new();
    let output = ctx
        .run(&["-m", "s2twp", "-t", "软件 鼠标 默认"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("軟體"));
    assert!(stdout.contains("滑鼠"));
    assert!(stdout.contains("預設"));
}

#[test]
fn text_s2hk() {
    let ctx = TestContext::new();
    let output = ctx.run(&["-m", "s2hk", "-t", "开"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("開"));
}

// --- File mode ---

#[test]
fn single_file_stdout() {
    let ctx = TestContext::new();
    ctx.create_file("input.txt", "开放中文转换");
    let output = ctx
        .run(&[
            "-m",
            "s2t",
            "-f",
            ctx.tmpdir.join("input.txt").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("開放中文轉換"));
}

#[test]
fn single_file_output() {
    let ctx = TestContext::new();
    ctx.create_file("input.txt", "开放中文转换");
    let out = ctx.tmpdir.join("output.txt");
    ctx.run(&[
        "-m",
        "s2t",
        "-f",
        ctx.tmpdir.join("input.txt").to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ])
    .output()
    .unwrap();
    assert_eq!(ctx.read_file("output.txt"), "開放中文轉換");
}

#[test]
fn multi_file_output() {
    let ctx = TestContext::new();
    ctx.create_file("a.txt", "开放");
    ctx.create_file("b.md", "转换");
    let out_dir = ctx.tmpdir.join("out");
    fs::create_dir_all(&out_dir).unwrap();

    ctx.run(&[
        "-m",
        "s2t",
        "-f",
        ctx.tmpdir.join("a.txt").to_str().unwrap(),
        "-f",
        ctx.tmpdir.join("b.md").to_str().unwrap(),
        "-o",
        out_dir.to_str().unwrap(),
    ])
    .output()
    .unwrap();

    assert_eq!(fs::read_to_string(out_dir.join("a.txt")).unwrap(), "開放");
    assert_eq!(fs::read_to_string(out_dir.join("b.md")).unwrap(), "轉換");
}

// --- Directory mode ---

#[test]
fn directory_mode() {
    let ctx = TestContext::new();
    ctx.create_file("sub/deep.txt", "软件和鼠标");
    let out_dir = ctx.tmpdir.join("output_dir");
    fs::create_dir_all(&out_dir).unwrap();

    ctx.run(&[
        "-m",
        "s2t",
        "-d",
        ctx.tmpdir.join("sub").to_str().unwrap(),
        "--ext",
        "txt",
        "-o",
        out_dir.to_str().unwrap(),
    ])
    .output()
    .unwrap();

    assert_eq!(
        fs::read_to_string(out_dir.join("deep.txt")).unwrap(),
        "軟件和鼠標"
    );
}

#[test]
fn directory_preserves_nested_structure() {
    let ctx = TestContext::new();
    ctx.create_file("src/nested/a.txt", "开放");
    let out_dir = ctx.tmpdir.join("out");
    fs::create_dir_all(&out_dir).unwrap();

    ctx.run(&[
        "-m",
        "s2t",
        "-d",
        ctx.tmpdir.join("src").to_str().unwrap(),
        "--ext",
        "txt",
        "-o",
        out_dir.to_str().unwrap(),
    ])
    .output()
    .unwrap();

    assert_eq!(
        fs::read_to_string(out_dir.join("nested/a.txt")).unwrap(),
        "開放"
    );
}

// --- In-place mode ---

#[test]
fn in_place_single_file() {
    let ctx = TestContext::new();
    ctx.create_file("inplace.txt", "开放中文转换");
    ctx.run(&[
        "-m",
        "s2t",
        "-f",
        ctx.tmpdir.join("inplace.txt").to_str().unwrap(),
        "--in-place",
    ])
    .output()
    .unwrap();
    assert_eq!(ctx.read_file("inplace.txt"), "開放中文轉換");
}

#[test]
fn in_place_directory() {
    let ctx = TestContext::new();
    ctx.create_file("d1/a.txt", "开放");
    ctx.create_file("d1/b.txt", "转换");
    ctx.run(&[
        "-m",
        "s2t",
        "-d",
        ctx.tmpdir.join("d1").to_str().unwrap(),
        "--ext",
        "txt",
        "--in-place",
    ])
    .output()
    .unwrap();
    assert_eq!(ctx.read_file("d1/a.txt"), "開放");
    assert_eq!(ctx.read_file("d1/b.txt"), "轉換");
}

// --- Error cases ---

#[test]
fn error_unsupported_mode() {
    let ctx = TestContext::new();
    let output = ctx.run(&["-m", "xyz", "-t", "test"]).output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unsupported mode"));
    assert!(!output.status.success());
}

#[test]
fn error_file_not_found() {
    let ctx = TestContext::new();
    let output = ctx
        .run(&["-m", "s2t", "-f", "/nonexistent/path.txt"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("file not found"));
    assert!(!output.status.success());
}

#[test]
fn error_inplace_with_text() {
    let ctx = TestContext::new();
    let output = ctx
        .run(&["-m", "s2t", "-t", "test", "--in-place"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--in-place requires"));
    assert!(!output.status.success());
}

#[test]
fn error_inplace_with_output() {
    let ctx = TestContext::new();
    let output = ctx
        .run(&[
            "-m",
            "s2t",
            "-f",
            "/tmp/test.txt",
            "--in-place",
            "-o",
            "/tmp/out.txt",
        ])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("mutually exclusive"));
    assert!(!output.status.success());
}

#[test]
fn error_same_file_without_inplace() {
    let ctx = TestContext::new();
    ctx.create_file("same.txt", "test");
    let path = ctx.tmpdir.join("same.txt");
    let output = ctx
        .run(&[
            "-m",
            "s2t",
            "-f",
            path.to_str().unwrap(),
            "-o",
            path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("input and output are the same"));
    assert!(!output.status.success());
}

#[test]
fn error_multi_file_no_output() {
    let ctx = TestContext::new();
    ctx.create_file("a.txt", "a");
    ctx.create_file("b.txt", "b");
    let output = ctx
        .run(&[
            "-m",
            "s2t",
            "-f",
            ctx.tmpdir.join("a.txt").to_str().unwrap(),
            "-f",
            ctx.tmpdir.join("b.txt").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("multiple files require"));
    assert!(!output.status.success());
}
