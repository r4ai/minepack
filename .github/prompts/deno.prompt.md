# Deno

これは Deno でのみ意識すべきマニュアルです。

## Guideline

### Important

- TypeScript で記述する
- `any`型は避け、方安全なプログラムとなるように努める  
  型がわからない場合は、`unknown`などを有効活用する
- 関数型プログラミングを意識し、シンプルで副作用のないプログラムになるように努める
- コードを変更した後は必ず静的検査を行い、品質保証に努める
- 機能を追加したりした場合にはテストを追加し、パスすることを確認する  
  パスしない場合は、実装が正しいかを入念に確認する

### Packages

URL インポートではなく JSR を利用する。

例：

```ts
// OK
import { join } from "jsr:@std/path"; // JSR import
import { hoge } from "npm:@hoge/hoge"; // NPM import (JSRにないツールを使う場合に使用する)

// NG
import { copy } from "https://deno.land/std@0.224.0/fs/copy.ts";
```

## Useful Libraries

### @std

- [**@std/assert**](https://jsr.io/@std/assert)  
  テスト時に役立つ一般的なアサーション関数群

- [**@std/async**](https://jsr.io/@std/async)  
  非同期処理用ユーティリティ（遅延、デバウンス、プールなど）

- [**@std/bytes**](https://jsr.io/@std/bytes)  
  JavaScript 標準にはない Uint8Array 操作ユーティリティ

- [**@std/cache**](https://jsr.io/@std/cache)（不安定）  
  キャッシュ操作ユーティリティ

- [**@std/cbor**](https://jsr.io/@std/cbor)（不安定）  
  CBOR（Concise Binary Object Representation）の解析とシリアライズ

- [**@std/cli**](https://jsr.io/@std/cli)  
  対話型 CLI ツール作成用ユーティリティ

- [**@std/collections**](https://jsr.io/@std/collections)  
  配列やオブジェクトなどコレクション型を扱う純粋関数群

- [**@std/crypto**](https://jsr.io/@std/crypto)  
  Web Crypto API の拡張機能

- [**@std/csv**](https://jsr.io/@std/csv)  
  CSV ファイルの読み書き

- [**@std/data-structures**](https://jsr.io/@std/data-structures)  
  赤黒木やバイナリヒープなどの一般的なデータ構造

- [**@std/datetime**](https://jsr.io/@std/datetime)（不安定）  
  日付・時間処理ユーティリティ

- [**@std/dotenv**](https://jsr.io/@std/dotenv)（不安定）  
  `.env`ファイルから環境変数を読み込むためのユーティリティ

- [**@std/encoding**](https://jsr.io/@std/encoding)  
  Hex、Base64、Varint など一般的な形式のエンコード・デコード

- [**@std/expect**](https://jsr.io/@std/expect)  
  Jest 互換のアサーション関数群

- [**@std/fmt**](https://jsr.io/@std/fmt)  
  値のフォーマット（色付け、期間フォーマット、printf 系ユーティリティ、バイト数の整形など）

- [**@std/front-matter**](https://jsr.io/@std/front-matter)  
  文字列からフロントマターを抽出

- [**@std/fs**](https://jsr.io/@std/fs)  
  ファイルシステム操作の補助関数群

- [**@std/html**](https://jsr.io/@std/html)  
  HTML エンティティのエスケープ・アンエスケープ

- [**@std/http**](https://jsr.io/@std/http)  
  HTTP サーバー構築のためのユーティリティ

- [**@std/ini**](https://jsr.io/@std/ini)（不安定）  
  INI ファイルの解析とシリアライズ

- [**@std/internal**](https://jsr.io/@std/internal)（内部用）  
  内部パッケージ（直接使用不可）

- [**@std/io**](https://jsr.io/@std/io)（不安定）  
  Reader と Writer インターフェースを用いた高度な I/O 操作用ユーティリティ

- [**@std/json**](https://jsr.io/@std/json)  
  JSON ファイルの（ストリーミング）解析とシリアライズ

- [**@std/jsonc**](https://jsr.io/@std/jsonc)  
  JSONC ファイルの解析とシリアライズ

- [**@std/log**](https://jsr.io/@std/log)（不安定）  
  カスタマイズ可能なロガーフレームワーク

- [**@std/media-types**](https://jsr.io/@std/media-types)  
  メディアタイプ（MIME タイプ）操作ユーティリティ

- [**@std/msgpack**](https://jsr.io/@std/msgpack)  
  MessagePack フォーマットのエンコードとデコード

- [**@std/net**](https://jsr.io/@std/net)  
  ネットワーク操作ユーティリティ

- [**@std/path**](https://jsr.io/@std/path)  
  ファイルシステムのパス操作ユーティリティ

- [**@std/random**](https://jsr.io/@std/random)（不安定）  
  ランダム値生成およびシード付き疑似乱数生成ユーティリティ

- [**@std/regexp**](https://jsr.io/@std/regexp)  
  正規表現操作ユーティリティ

- [**@std/semver**](https://jsr.io/@std/semver)  
  セマンティックバージョニング（SemVer）の解析と比較

- [**@std/streams**](https://jsr.io/@std/streams)  
  Web Streams API 操作用ユーティリティ

- [**@std/tar**](https://jsr.io/@std/tar)（不安定）  
  tar アーカイブのストリーミング処理ユーティリティ

- [**@std/testing**](https://jsr.io/@std/testing)  
  スナップショットテスト、BDD テスト、時間モックなど Deno コードテスト用ツール群

- [**@std/text**](https://jsr.io/@std/text)  
  テキスト処理ユーティリティ

- [**@std/toml**](https://jsr.io/@std/toml)  
  TOML ファイルの解析とシリアライズ

- [**@std/ulid**](https://jsr.io/@std/ulid)  
  ULID の生成

- [**@std/uuid**](https://jsr.io/@std/uuid)  
  UUID の生成と検証

- [**@std/webgpu**](https://jsr.io/@std/webgpu)（不安定）  
  WebGPU API 操作ユーティリティ

- [**@std/yaml**](https://jsr.io/@std/yaml)  
  YAML ファイルの解析とシリアライズ

使用方法：

```ts
import { join } from "@std/path";
```

### dax

Deno および Node.js 向けのクロスプラットフォームシェルツール。コマンド実行、HTTP リクエスト、ファイル操作、プロンプト機能などを提供。

使用方法：

```ts
import $ from "jsr:@david/dax";

// run a command
await $`echo 5`; // outputs: 5

// outputting to stdout and running a sub process
await $`echo 1 && deno run main.ts`;

// parallel
await Promise.all([
  $`sleep 1 ; echo 1`,
  $`sleep 2 ; echo 2`,
  $`sleep 3 ; echo 3`,
]);
```

詳細な使用方法は https://raw.githubusercontent.com/dsherret/dax/refs/heads/main/README.md を参照のこと。

## Static Analysis

品質保証に使える静的解析として、以下があります。

- `deno fmt hoge.ts`: Format
- `deno lint hoge.ts`: Lint
- `deno check hoge.ts`: 型検査

ファイルに変更を加えた後は、必ずこれらを実行することで品質保証に努めてください。
