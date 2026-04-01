# opencc-cli

A command-line tool for converting Chinese text between Simplified and Traditional using [OpenCC](https://github.com/BYVoid/OpenCC).

Built with Rust and [ferrous-opencc](https://crates.io/crates/ferrous-opencc) (pure Rust, no system dependencies).

## Install

```bash
cargo build --release
```

Binary: `target/release/opencc-cli`

## Usage

### Text

```bash
opencc-cli -m s2t -t "开放中文转换"
# → 開放中文轉換
```

### Single file (stdout)

```bash
opencc-cli -m s2t -f input.txt
```

### Single file (output)

```bash
opencc-cli -m s2t -f input.txt -o output.txt
```

### Multiple files

```bash
opencc-cli -m s2t -f a.txt -f b.md -o out/
```

### Directory (recursive)

```bash
opencc-cli -m s2t -d ./src -o ./out --ext txt,md
```

### In-place overwrite

```bash
opencc-cli -m s2t -f input.txt --in-place
opencc-cli -m s2t -d ./docs --in-place --ext txt,md
```

### Pipe (stdin)

```bash
echo "汉字" | opencc-cli -m s2t
# → 漢字
```

## Options

| Option | Description |
|--------|-------------|
| `-m, --mode` | Conversion mode (default: `s2t`) |
| `-t, --text` | Direct text input |
| `-f, --file` | Input file(s), can specify multiple times |
| `-d, --dir` | Input directory (recursive) |
| `-o, --output` | Output path (file or directory) |
| `--ext` | Extension filter for directory mode (default: `txt,md,csv,json,xml,html,yaml,yml`) |
| `--in-place` | Overwrite original files |

## Conversion Modes

| Mode | Direction |
|------|-----------|
| `s2t` | Simplified → Traditional |
| `t2s` | Traditional → Simplified |
| `s2tw` | Simplified → Traditional (Taiwan) |
| `tw2s` | Traditional (Taiwan) → Simplified |
| `s2hk` | Simplified → Traditional (Hong Kong) |
| `hk2s` | Traditional (Hong Kong) → Simplified |
| `s2twp` | Simplified → Traditional (Taiwan, with vocabulary) |
| `tw2sp` | Traditional (Taiwan, with vocabulary) → Simplified |
| `t2tw` | Traditional → Traditional (Taiwan) |
| `tw2t` | Traditional (Taiwan) → Traditional |
| `t2hk` | Traditional → Traditional (Hong Kong) |
| `hk2t` | Traditional (Hong Kong) → Traditional |
| `t2jp` | Traditional → Japanese Shinjitai |
| `jp2t` | Japanese Shinjitai → Traditional |

## Rules

- `-t`, `-f`, `-d` are mutually exclusive input sources
- `-o` and `--in-place` are mutually exclusive
- Multiple files (`-f` specified more than once) require `-o` or `--in-place`
- Directory mode preserves relative path structure in output
- Only UTF-8 encoded files are supported

## Test

```bash
cargo test
```
