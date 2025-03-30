## 品質保証

コードに変更を加えた後は、必ず次のコマンドをすべて実行して、エラーが無いことを確認してください。

- `cargo fmt --all`
- `cargo clippy --fix --allow-dirty`
- `cargo clippy -- -D warnings`
- `cargo test`
- `cargo build`
