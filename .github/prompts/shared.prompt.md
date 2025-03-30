# Shared

これはすべての言語で共通して意識すべきマニュアルです。

## Git

コミットメッセージは、Conventional Commits に従って英語で記述してください。

## Mise

次に Mise を利用しています：

- 開発に利用するツールの管理
- タスクランナー

使用例：

```sh
# install tools
mise install

# set tool version
mise use rust@1.85.1 nm,

# run tasks
mise tasks run test  # Testを実行する
```
