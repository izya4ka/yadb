# YADB - Yet Another Directory Buster
![Issues](https://img.shields.io/github/issues/izya4ka/yadb)
![Last Commit](https://img.shields.io/github/last-commit/izya4ka/yadb)
![](https://img.shields.io/crates/l/yadb)
![](https://img.shields.io/github/languages/top/izya4ka/yadb)
[![Built With Ratatui](https://ratatui.rs/built-with-ratatui/badge.svg)](https://ratatui.rs/)
![WindowsTerminal_XzDicVjS7F-ezgif com-cut](https://github.com/user-attachments/assets/45368b2d-0616-40e4-9eec-5fb33ab9d9b6)
![ezgif-71158575d9683e](https://github.com/user-attachments/assets/f1fd7a50-4aa0-4c4a-a438-a22dd5b5be23)


**YADB** is a directory brute-forcing tool written in **Rust**, inspired by `gobuster`.

## ✨ Features
- ⚡ **High performance** with multithreading
- 🖥️ **CLI and TUI interface**

## 📦 Installation
```bash
cargo install yadb
```

## 🚀 Usage

### CLI

```
Usage: yadb-cli [OPTIONS] --wordlist <WORDLIST> --uri <URI>

Options:
  -t, --threads <THREADS>      Number of threads [default: 50]
  -r, --recursive <RECURSIVE>  Recursivly parse directories and files (recursion depth) [default: 0]
  -w, --wordlist <WORDLIST>    Path to wordlist
  -u, --uri <URI>              Target URI
  -o, --output <OUTPUT>        Output file
  -h, --help                   Print help
  -V, --version                Print version
```

### TUI
Just simply:
```
yadb-tui
```

## 🛠️ TODO

- [x] CLI interface
- [x] Recursion
- [x] TUI interface
- [ ] Output in HTML/JSON formats
- [ ] Better TUI :)

## 🙌 Contributions
Contributions are welcome! If you have ideas for improvements, bug fixes, or new features, feel free to open an issue or submit a pull request.

## 📄 License

This project is licensed under the **GNU General Public License version 3**.

## ⚠️ Disclaimer

This project is provided for educational and research purposes only — use it responsibly and only on systems you own or have explicit permission to test; the author accepts no liability for any misuse or damage.
