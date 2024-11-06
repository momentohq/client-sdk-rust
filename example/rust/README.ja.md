# Rust クライアント SDK

_他言語バージョンもあります_：[English](README.md)

<br>

## SDK のコード例を実行する

- Rust と Cargo が必要です。 [インストレーションガイド](https://doc.rust-lang.org/cargo/getting-started/installation.html)。
- Momento オーストークンが必要です。トークン発行は[Momento CLI](https://github.com/momentohq/momento-cli)から行えます。

```bash
# SDKコード例を実行する
MOMENTO_API_KEY=<YOUR API KEY> cargo run --bin=cache
```

SDK コード例: [cache.rs](src/bin/cache.rs)

## SDK を自身のプロジェクトで使用する

`momento = "0.3.1"`をご自身の Cargo.toml ファイルに追加してください。もしくは最新バージョンを[こちら](https://crates.io/crates/momento)からご確認下さい。
