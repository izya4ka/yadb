# YADB - Yet Another Directory Buster
![Issues](https://img.shields.io/github/issues/izya4ka/yadb)
![Last Commit](https://img.shields.io/github/last-commit/izya4ka/yadb)
![](https://img.shields.io/crates/l/yadb)
![](https://img.shields.io/github/languages/top/izya4ka/yadb)

![WindowsTerminal_XzDicVjS7F-ezgif com-cut](https://github.com/user-attachments/assets/45368b2d-0616-40e4-9eec-5fb33ab9d9b6)


**YADB** is a fast and safe directory brute-forcing tool written in **Rust**, inspired by `gobuster`.

## ✨ Features
- ⚡ **High performance** with multithreading
- 🖥️ **CLI interface** (GUI — coming soon)
- 📝 **Logging** to file and stdout
- 📊 **Progress bar** for real-time feedback
- 🔒 **Safety** — robust error handling and thread safety

## 📦 Installation
```bash
cargo install yadb
```

## 🚀 Usage

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

## 🛠️ TODO

- [x] CLI interface
- [x] Recursion
- [ ] GUI interface (planned, using egui or iced)
- [ ] Output in HTML/JSON formats
- [ ] Automatic wordlist updates

## 🙌 Contributions
Contributions are welcome! If you have ideas for improvements, bug fixes, or new features, feel free to open an issue or submit a pull request.

## 📄 License

This project is licensed under the **GNU General Public License version 3**.
