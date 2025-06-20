# YADB - Yet Another Directory Buster
![Issues](https://img.shields.io/github/issues/izya4ka/yadb)
![Last Commit](https://img.shields.io/github/last-commit/izya4ka/yadb)
![](https://img.shields.io/crates/l/yadb)
![](https://img.shields.io/github/languages/top/izya4ka/yadb)



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
yadb-cli [OPTIONS] --wordlist <WORDLIST> --uri <URI>

Options:
  -t, --threads <THREADS>    Number of threads [default: 50]
  -r, --recursive            Recursivly parse directories and files (TODO!)
  -w, --wordlist <WORDLIST>  Path to wordlist
  -u, --uri <URI>            Target URI
  -o, --output <OUTPUT>      Output file
  -h, --help                 Print help
  -V, --version              Print version
```

## 🛠️ TODO

- [x] CLI interface
- [ ] GUI interface (planned, using egui or iced)
- [ ] Output in HTML/JSON formats
- [ ] Automatic wordlist updates

## 📄 License

This project is licensed under the **GNU General Public License version 3**.