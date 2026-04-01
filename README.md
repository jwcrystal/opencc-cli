# opencc-cli

Command-line tool for Chinese text conversion between Simplified and Traditional.

Supports 14 conversion modes including Simplified ↔ Traditional, Taiwan ↔ Simplified, Hong Kong ↔ Simplified, and Japanese Shinjitai.

Built with [OpenCC](https://github.com/BYVoid/OpenCC) via [ferrous-opencc](https://crates.io/crates/ferrous-opencc) — pure Rust, no system dependencies.

## Install

```bash
cargo build --release
```

Binary: `target/release/opencc-cli`

## Usage

### Text

```bash
opencc-cli -m s2t -t "开放中文转换"
# Output: 開放中文轉換
```

### Pipe (stdin)

```bash
echo "汉字" | opencc-cli -m s2t
# Output: 漢字
```

### Single file

```bash
opencc-cli -m s2t -f input.txt              # stdout
opencc-cli -m s2t -f input.txt -o output.txt  # write to file
```

### Multiple files

```bash
opencc-cli -m s2t -f a.txt -f b.txt -o out/
```

### Directory (recursive)

```bash
opencc-cli -m s2t -d ./folder -o output_folder/   # preserves structure
opencc-cli -m s2t -d ./folder --ext txt,md,csv     # filter by extension
```

### In-place overwrite

```bash
opencc-cli -m s2t -f input.txt --in-place
opencc-cli -m s2twp -d ./src --in-place
```

## Supported Modes

| Mode | Direction |
|------|-----------|
| `s2t` | Simplified → Traditional |
| `t2s` | Traditional → Simplified |
| `s2tw` | Simplified → Traditional (Taiwan) |
| `tw2s` | Traditional (Taiwan) → Simplified |
| `s2hk` | Simplified → Traditional (Hong Kong) |
| `hk2s` | Traditional (Hong Kong) → Simplified |
| `s2twp` | Simplified → Traditional (Taiwan, with phrases) |
| `tw2sp` | Traditional (Taiwan, with phrases) → Simplified |
| `t2tw` | Traditional → Traditional (Taiwan) |
| `tw2t` | Traditional (Taiwan) → Traditional |
| `t2hk` | Traditional → Traditional (Hong Kong) |
| `hk2t` | Traditional (Hong Kong) → Traditional |
| `t2jp` | Traditional → Japanese Shinjitai |
| `jp2t` | Japanese Shinjitai → Traditional |

## Options

| Flag | Default | Description |
|------|---------|-------------|
| `-m, --mode` | `s2t` | Conversion mode |
| `-t, --text` | — | Direct text input (mutually exclusive with `-f`/`-d`) |
| `-f, --file` | — | Input file(s), can repeat (mutually exclusive with `-t`/`-d`) |
| `-d, --dir` | — | Input directory, recursive (mutually exclusive with `-t`/`-f`) |
| `-o, --output` | — | Output path (file or directory) |
| `--ext` | `txt,md,csv,json,xml,html,yaml,yml` | Extension filter for directory mode |
| `--in-place` | — | Overwrite original files (incompatible with `-o`) |

When no input is provided, reads from stdin (pipe).

## Error Messages

```
error: unsupported mode 'xyz'. Available: s2t, t2s, ...
error: file not found: '/path/to/file'
error: input and output are the same file: 'path'. Use --in-place to overwrite.
error: --in-place and -o are mutually exclusive.
error: --in-place requires -f or -d input, not -t.
error: multiple files require -o <directory> or --in-place.
error: no matching files in 'folder' (--ext: txt,md,...)
error: invalid UTF-8: ...
```

## Rules

- `-t`, `-f`, `-d` are mutually exclusive input sources
- `-o` and `--in-place` are mutually exclusive
- Multiple files require `-o` or `--in-place`
- Directory mode preserves relative path structure in output
- Only UTF-8 encoded files are supported

## Test

```bash
cargo test
```

## License

Apache-2.0
