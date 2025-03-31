# Rust

これは Rust 言語でのみ意識すべきマニュアルです。

## 品質保証

コードに変更を加えた後は、必ず次のコマンドをすべて実行して、エラーが無いことを確認してください。

- `cargo fmt --all`
- `cargo clippy --fix --allow-dirty --allow-staged`
- `cargo clippy -- -D warnings`
- `mise tasks run test`
- `mise tasks run build`
