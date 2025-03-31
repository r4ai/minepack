# Shared

これはすべての言語で共通して意識すべきマニュアルです。

## Commands

開発に使用するコマンドとして以下があります。必要に応じて実行してください。

| command                       | description                        |
| ----------------------------- | ---------------------------------- |
| `mise tasks run format`       | フォーマットをチェックする         |
| `mise tasks run format-write` | フォーマットする                   |
| `mise tasks run lint`         | リントする                         |
| `mise tasks run lint-write`   | リントし、エラー個所を自動修正する |
| `mise tasks run build`        | ビルドする                         |
| `mise tasks run test`         | テストを実行する                   |

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
mise use rust@1.85.1

# run tasks
mise tasks run test
```
