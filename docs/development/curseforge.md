> **プロンプト**
>
> Curseforge の API について、使用方法を詳しくまとめてください。
> Minecraft の Mod の検索、自動インストール、Mod パックの取得などを行いたいです。
> 背景情報としては、Modpack を作成する CLI ツールを Rust で作成しているため、このツールから Curseforge にアクセスするために API について知りたいです。

# CurseForge API を用いた Minecraft Mod パック CLI ツール開発ガイド

Minecraft 向けの Mod パック作成 CLI ツールを Rust で開発する際に、**CurseForge 公式 API**を利用して以下の機能を実装する方法を解説します:

1. **Minecraft の Mod 検索** – キーワードや条件で Mod を検索
2. **Mod の自動インストール（ダウンロード URL 取得）** – 検索した Mod からダウンロードリンクを取得
3. **Mod パックの検索および取得** – Mod パックプロジェクトを検索し、ファイルをダウンロード

各機能に関連する API エンドポイントの使い方や、認証・パラメータ指定方法、Rust (`reqwest`クレート)からのリクエスト例、API 利用上の注意点（認証やレートリミット）について詳しく説明します。

## CurseForge API の概要と認証

**公式の CurseForge API**を利用するには、まず Overwolf（CurseForge の提供元）から**API キー**を取得する必要があります。API キーは開発者用のフォームから申請して発行してもらいます ([About the CurseForge API and How to Apply for a Key: CurseForge support](https://support.curseforge.com/en/support/solutions/articles/9000208346-about-the-curseforge-api-and-how-to-apply-for-a-key#:~:text=How%20to%20Apply%20for%20an,API%20Key))。発行されたキーは**`x-api-key`**という HTTP ヘッダーに含めてリクエスト毎に送信します ([Unable to download any curseforge mods after api switch · Issue #2045 · itzg/docker-minecraft-server · GitHub](https://github.com/itzg/docker-minecraft-server/issues/2045#:~:text=agent%3A%20mc,com))。API キーがないと全てのエンドポイントでアクセスが拒否されます。

- **API キー取得方法**: Overwolf の提供する申請フォームにプロジェクト情報などを入力して申請します。承認されるとメール等で一意の API キーが発行されます ([About the CurseForge API and How to Apply for a Key: CurseForge support](https://support.curseforge.com/en/support/solutions/articles/9000208346-about-the-curseforge-api-and-how-to-apply-for-a-key#:~:text=form%20and%20reviewed%20by%20the,Overwolf%20team))。
- **リクエスト時の認証**: リクエストヘッダーに `x-api-key: <発行されたキー>` を指定します ([Unable to download any curseforge mods after api switch · Issue #2045 · itzg/docker-minecraft-server · GitHub](https://github.com/itzg/docker-minecraft-server/issues/2045#:~:text=agent%3A%20mc,com))。また、必要に応じて`Accept: application/json`ヘッダーを付与し、JSON 形式でレスポンスを受け取ります。
- **基本 URL**: API のベース URL は **`https://api.curseforge.com`** です ([Getting Started – CurseForge for Studios API](https://docs.curseforge.com/rest-api/#:~:text=Accessing%20the%20service,API%20key%20can%20be))。全てのエンドポイントはこのベース URL に続くパスで指定します。
- **データ形式**: レスポンスは JSON 形式で返され、`"data"`フィールド以下に結果が格納されます。例えば一覧系エンドポイントでは`"data"`に配列が入り、単一リソース取得ではオブジェクトが入ります。

> **メモ:** CurseForge API は**Rate Limit（レート制限）**が設けられています。一度に大量のリクエストを送ると一時的にアクセス禁止（HTTP 403/429 エラー）となる場合があります ([API Key is rate-limited of there are manual mod packs to install · Issue #2647 · itzg/docker-minecraft-server · GitHub](https://github.com/itzg/docker-minecraft-server/issues/2647#:~:text=mc_helltime%20%20%7C%20%5Bmc,install%20CurseForge%20modpack)) ([API Key is rate-limited of there are manual mod packs to install · Issue #2647 · itzg/docker-minecraft-server · GitHub](https://github.com/itzg/docker-minecraft-server/issues/2647#:~:text=itzg%20%20%20commented%20,74))。具体的な上限値は公表されていませんが、短時間に多数のリクエストを送信しないよう注意してください（後述の「API 制限」に詳細）。

## Minecraft の Mod 検索方法

CurseForge API を使うことで、Minecraft の多数の Mod プロジェクトを検索することができます。検索には**GET**リクエストで**`/v1/mods/search`**エンドポイントを使用します。主なクエリパラメータと指定方法は以下の通りです。

- **gameId** – ゲームの ID。Minecraft の場合は **`432`** を指定します ([GitHub - aternosorg/php-curseforge-api: PHP Client for the CurseForge API.](https://github.com/aternosorg/php-curseforge-api#:~:text=You%20can%20also%20call%20,is%20the%20ID%20for%20Minecraft))（必須）。
- **classId** – カテゴリクラス ID（任意）。Minecraft では「Mods」クラスは **`6`**、「Modpacks（Mod パック）」クラスは **`4471`** に対応します ([curse_api.md · GitHub](https://gist.github.com/crapStone/9a423f7e97e64a301e88a2f6a0f3e4d9#:~:text=match%20at%20L244%204471%20Modpacks))。Mods のみを検索する場合は`classId=6`を指定します（指定しない場合、Modpacks や他のカテゴリも含む全プロジェクトが検索対象になります）。
- **categoryId** – カテゴリ ID（任意）。特定のサブカテゴリで絞り込む場合に指定します（例：「World Generation（地形生成）」カテゴリの ID など）。`classId`で大分類を指定し、`categoryId`で小分類を指定できます ([GitHub - aternosorg/php-curseforge-api: PHP Client for the CurseForge API.](https://github.com/aternosorg/php-curseforge-api#:~:text=Game%20Categories))。
- **searchFilter** – 検索キーワード（任意）。Mod 名や概要、作者名に対するフリーテキスト検索語を指定します ([php-curseforge-api/openapi.yaml at master · aternosorg/php-curseforge-api · GitHub](https://github.com/aternosorg/php-curseforge-api/blob/master/openapi.yaml#:~:text=))。例：`searchFilter=JEI`（「JEI」という単語を含む Mod を検索）。
- **gameVersion** – ゲームのバージョン（任意）。対応する Minecraft バージョンで絞り込みます ([Interface SearchOptions | X Minecraft Launcher](https://www.xmcl.app/en/core/curseforge/SearchOptions#:~:text=gameVersion%3A%20string))。文字列で指定し、例：`gameVersion=1.19.2`。
- **modLoaderType** – Mod ローダーの種別（任意）。Forge や Fabric など Mod の対応プラットフォームで絞り込みます ([php-curseforge-api/openapi.yaml at master · aternosorg/php-curseforge-api · GitHub](https://github.com/aternosorg/php-curseforge-api/blob/master/openapi.yaml#:~:text=description%3A%20))。数値の Enum 値で指定し、**このパラメータを使う場合は`gameVersion`も併せて指定**する必要があります ([php-curseforge-api/openapi.yaml at master · aternosorg/php-curseforge-api · GitHub](https://github.com/aternosorg/php-curseforge-api/blob/master/openapi.yaml#:~:text=description%3A%20))。たとえば Forge 用 Mod のみ探す場合や、Fabric 用 Mod のみ探す場合に利用します（Enum 値の詳細は公式ドキュメント参照）。
- **sortField**・**sortOrder** – ソート指定（任意）。`sortField`にはソート基準の Enum 値（人気順、更新日順など）を指定し ([php-curseforge-api/openapi.yaml at master · aternosorg/php-curseforge-api · GitHub](https://github.com/aternosorg/php-curseforge-api/blob/master/openapi.yaml#:~:text=))、`sortOrder`には`asc`(昇順)か`desc`(降順)を指定します ([php-curseforge-api/openapi.yaml at master · aternosorg/php-curseforge-api · GitHub](https://github.com/aternosorg/php-curseforge-api/blob/master/openapi.yaml#:~:text=))。デフォルトでは更新日の降順など決められた順序で結果が返ります。
- **index**・**pageSize** – ページネーション用（任意）。`pageSize`は 1 ページあたりの件数（最大 50 件）、`index`は 0 始まりのオフセットを指定します。※検索結果は最大 10,000 件までしか取得できません（`index + pageSize <= 10000`の範囲内で指定） ([php-curseforge-api/openapi.yaml at master · aternosorg/php-curseforge-api · GitHub](https://github.com/aternosorg/php-curseforge-api/blob/master/openapi.yaml#:~:text=))。

以上のパラメータを組み合わせて Mod 検索が可能です。**gameId=432**（Minecraft）は必須で、他のフィルタは必要に応じて指定します（何も指定しないと Minecraft の全プロジェクトが返るため、通常は`classId`や`searchFilter`を指定します ([GitHub - CurseForgeCommunity/.NET-APIClient: A CurseForge API Client (For CurseForge Core)](https://github.com/CurseForgeCommunity/.NET-APIClient#:~:text=Requires%20at%20least%20one%20filter,to%20be%20filled%20in))）。

**例:** _「Minecraft 1.19.2 対応の Fabric 用 JEI(Mod 名)」を検索するリクエスト例_

```rust
use reqwest::blocking::Client;

let api_key = "YOUR_API_KEY";  // 取得したAPIキーをセット
let query = "JEI";
let game_version = "1.19.2";
let mod_loader_type = 4;  // 例: Fabricを示すModLoaderTypeの値 (Forge=1, Fabric=4 等)

let url = format!(
    "https://api.curseforge.com/v1/mods/search?gameId=432&classId=6&searchFilter={}&gameVersion={}&modLoaderType={}",
    query, game_version, mod_loader_type
);
let client = Client::new();
let res_text = client
    .get(&url)
    .header("x-api-key", api_key)
    .header("Accept", "application/json")
    .send()
    .expect("Failed to send request")
    .text()
    .expect("Failed to read response text");

println!("{}", res_text);
```

上記のように`reqwest`クレートを用いて GET リクエストを送信できます。`res_text`には JSON 形式の文字列が入り、検索結果の Mod 一覧が含まれます。例えばレスポンスの一部は以下のような形式です（簡略化）:

```json
{
  "data": [
    {
      "id": 238222,
      "name": "Just Enough Items (JEI)",
      "summary": "...JEI description...",
      "slug": "jei",
      "links": { ... },
      "latestFiles": [ ... ],
      "categories": [ ... ],
      "gameId": 432,
      "classId": 6,
      "authors": [ {"name": "Mezz", ...} ],
      ...
    },
    { ... 次のMod ... }
  ],
  "pagination": { "index": 0, "pageSize": 50, "resultCount": 1, "totalCount": 1 }
}
```

各 Mod エントリには`id`（Mod のプロジェクト ID）、`name`（名称）、`summary`（概要）、`classId`や`gameId`、`authors`（作者情報）、`categories`（属しているカテゴリ）など様々な情報が含まれています。**Mod の ID (`id`)**は後続の詳細取得やダウンロードで必要になる重要な値です。

> 🔍 **カテゴリ ID の調べ方:** 特定カテゴリのみ検索したい場合、事前に**`/v1/categories`**エンドポイントで Minecraft のカテゴリ一覧を取得できます。例えば`/v1/categories?gameId=432`とすると、Minecraft の全カテゴリクラスおよびカテゴリの一覧が取得できま ([GitHub - aternosorg/php-curseforge-api: PHP Client for the CurseForge API.](https://github.com/aternosorg/php-curseforge-api#:~:text=Game%20Categories))】。返される各カテゴリオブジェクトには`id`（カテゴリ ID）、`classId`（属する大分類の ID）、`name`（名称）などが含まれます。それらを参照し、検索時に`classId`や`categoryId`パラメータとして利用することが可能です。

## Mod の自動インストール（ファイル取得とダウンロード）

検索して Mod の**プロジェクト ID** (`modId`)が分かったら、その Mod のファイル一覧から目的のバージョンをダウンロードできます。CurseForge 上の各 Mod プロジェクトは Minecraft のバージョンごとや Mod ローダー（Forge/Fabric 等）ごとに複数のファイル（バージョン違いの Mod 本体）を持っています。目的のファイルを特定し、**ダウンロード URL**を取得するまでの一般的な手順は次のとおりです。

1. **Mod 詳細の取得（任意）** – エンドポイント: `GET /v1/mods/{modId}`  
   Mod の詳細情報を取得します。これには Mod 名、説明、作者、ダウンロード数などが含まれま ([php-curseforge-api/openapi.yaml at master · aternosorg/php-curseforge-api · GitHub](https://github.com/aternosorg/php-curseforge-api/blob/master/openapi.yaml#:~:text=%2Fv1%2Fmods%2F)) ([php-curseforge-api/openapi.yaml at master · aternosorg/php-curseforge-api · GitHub](https://github.com/aternosorg/php-curseforge-api/blob/master/openapi.yaml#:~:text=parameters%3A))】。このレスポンスには`latestFiles`（最新ファイルの簡易情報リスト）や`gameVersionLatestFiles`（ゲームバージョンごとの最新ファイル情報）といったフィールドも含まれ、簡易的に最新の対応ファイル ID を知ることも可能です。ただし特定のファイルを得るには次のファイル一覧取得が確実です。

2. **Mod のファイル一覧取得** – エンドポイント: `GET /v1/mods/{modId}/files`  
   指定した Mod プロジェクトが持つすべてのファイル情報を取得しま ([php-curseforge-api/openapi.yaml at master · aternosorg/php-curseforge-api · GitHub](https://github.com/aternosorg/php-curseforge-api/blob/master/openapi.yaml#:~:text=tags%3A)) ([php-curseforge-api/openapi.yaml at master · aternosorg/php-curseforge-api · GitHub](https://github.com/aternosorg/php-curseforge-api/blob/master/openapi.yaml#:~:text=parameters%3A))】。クエリパラメータで`gameVersion`や`modLoaderType`を指定すると、その条件に合致するファイルだけに絞り込むこともできま ([php-curseforge-api/openapi.yaml at master · aternosorg/php-curseforge-api · GitHub](https://github.com/aternosorg/php-curseforge-api/blob/master/openapi.yaml#:~:text=)) ([php-curseforge-api/openapi.yaml at master · aternosorg/php-curseforge-api · GitHub](https://github.com/aternosorg/php-curseforge-api/blob/master/openapi.yaml#:~:text=))】。返される JSON には各ファイルごとに`id`（**fileId**）、`displayName`（ファイル名）、対応する Minecraft バージョン（`gameVersions`配列）、対応 Mod ローダー（`modLoader`フィールド）などが含まれます。

   **例:** `GET /v1/mods/238222/files?gameVersion=1.19.2&modLoaderType=4`  
   上記は Mod ID 238222 (JEI) のファイル一覧から「Minecraft 1.19.2」かつ「Fabric 用（modLoaderType=4）」のファイルに絞って取得する例です。結果として該当する JEI の Fabric 版ファイル（例えば「jei-1.19.2-fabric-x.y.z.jar」）の情報が得られ、その中の`id`がそのファイルの ID になります。

3. **ダウンロード URL の取得** – エンドポイント: `GET /v1/mods/{modId}/files/{fileId}/download-url`  
   ダウンロードしたいファイル ID が分かったら、このエンドポイントで**直リンク URL**を取得しま ([php-curseforge-api/openapi.yaml at master · aternosorg/php-curseforge-api · GitHub](https://github.com/aternosorg/php-curseforge-api/blob/master/openapi.yaml#:~:text=))】。成功するとレスポンスの`"data"`フィールドに一時的なダウンロード用 URL が含まれます。例えば次のような JSON が返ります（URL は一例）:

   ```json
   {
     "data": "https://edge.forgecdn.net/files/3458/765/modname-1.2.3.jar"
   }
   ```

   （※実際には`data`がオブジェクトで `{ "downloadUrl": "..." }` のような形式になる可能性がありますが、公式ドキュメント上は単に URL 文字列が返るとされています。）

   この取得した URL に対してさらに HTTP リクエストを送ることで、Mod 本体（JAR ファイル）をダウンロードできます。上記 URL は CurseForge の CDN から直接ファイルを取得するリンクです。

上記の操作を Rust コードで行う場合、`reqwest`で連続的に呼び出すことができます。簡単な例を示します。

```rust
use serde_json::Value;
use reqwest::blocking::Client;

let api_key = "YOUR_API_KEY";
let mod_id = 238222;      // 例: JEIのMod ID
let file_id = 4021230;    // 例: ダウンロードしたいファイルのID（JEIの特定バージョン）

let client = Client::new();

// 1. Mod詳細取得（必要に応じて）:
let mod_url = format!("https://api.curseforge.com/v1/mods/{}", mod_id);
let mod_resp: Value = client
    .get(&mod_url)
    .header("x-api-key", api_key)
    .send().unwrap()
    .json().unwrap();
println!("Mod Name: {}", mod_resp["data"]["name"]);  // Mod名など利用可能

// 2. ファイル一覧取得（ここでは省略し、file_idが既知と仮定）
//    必要なら /mods/{modId}/files にGETし、file_idを探索する

// 3. ダウンロードURL取得:
let dl_url_endpoint = format!("https://api.curseforge.com/v1/mods/{}/files/{}/download-url", mod_id, file_id);
let dl_resp: Value = client
    .get(&dl_url_endpoint)
    .header("x-api-key", api_key)
    .send().unwrap()
    .json().unwrap();

let download_url = dl_resp["data"].as_str().unwrap();  // 取得したダウンロードリンク
println!("Download URL: {}", download_url);

// 4. 実際のファイルをダウンロード:
let file_bytes = client.get(download_url).send().unwrap().bytes().unwrap();
// file_bytesにModファイル(JAR)の中身がバイト列で入るので、あとは保存する等の処理
```

上記コードでは、まず Mod 情報を取得し（省略可能）、既知の`mod_id`と`file_id`からダウンロード URL を取得しています。その後、その URL に対して再度`reqwest`で GET を行い、`bytes()`でバイト列を取得しています。あとは任意のパスに書き出すことで JAR ファイルを保存できます。

**補足:** ファイル一覧取得で目的の`fileId`を探す際、`gameVersion`や`modLoaderType`でフィルタしておけば、対象のファイルをプログラム的に選択しやすくなりま ([php-curseforge-api/openapi.yaml at master · aternosorg/php-curseforge-api · GitHub](https://github.com/aternosorg/php-curseforge-api/blob/master/openapi.yaml#:~:text=)) ([php-curseforge-api/openapi.yaml at master · aternosorg/php-curseforge-api · GitHub](https://github.com/aternosorg/php-curseforge-api/blob/master/openapi.yaml#:~:text=))】。たとえば最新版を取る場合は、ファイル一覧を更新日時でソートして先頭を選ぶか、`/mods/{modId}`のレスポンスに含まれる`latestFiles`から目的のバージョンを探す方法もあります。

## Mod パックの検索と取得

**Mod パック**も基本的には**Mod と同じ API**で管理されています。CurseForge 上では、Mod パックも一つの「プロジェクト（Mod と同列の扱い）」として提供されており、**クラス ID で Mod パックを指定**することで検索・取得が可能です。

### Mod パックの検索

Mod パックを検索するには、前述の`/v1/mods/search`エンドポイントで**`classId=4471`**（Minecraft における「Modpacks」クラスの ID）を指定しま ([curse_api.md · GitHub](https://gist.github.com/crapStone/9a423f7e97e64a301e88a2f6a0f3e4d9#:~:text=match%20at%20L244%204471%20Modpacks))】。その他のパラメータ（`gameId=432`や`searchFilter`など）は Mods 検索時と同様に利用できます。例えば、特定のキーワードで Mod パック名を検索したり、カテゴリー ID で絞り込むことも可能です。

- **例:** Minecraft の Mod パックをキーワード「Sky」により検索:  
  `GET https://api.curseforge.com/v1/mods/search?gameId=432&classId=4471&searchFilter=Sky`

上記のようにクラス ID を 4471 にすることで、検索結果は Minecraft の Mod パックに限定されます。レスポンス形式も Mods 検索と同様で、`data`配列に各プロジェクト（Mod パック）の情報が入ります。それぞれの`id`（Mod パック ID）を取得しておきます。

### Mod パックの取得・ダウンロード

Mod パックも通常の Mod と同様に**ファイル一覧**と**ダウンロード URL**を取得できます。実際の手順は Mods の場合と同じです。

1. **Mod パック詳細情報の取得**（任意） – `GET /v1/mods/{modpackId}`で Mod パックの情報を取得できます。
2. **Mod パックのファイル一覧取得** – `GET /v1/mods/{modpackId}/files`で、その Mod パックのリリースファイル一覧を取得します。通常 Mod パックはバージョン更新ごとに配布ファイル（ZIP ファイル）が作られます。必要に応じて`gameVersion`等でフィルタ可能です。
3. **ファイルのダウンロード URL 取得** – `GET /v1/mods/{modpackId}/files/{fileId}/download-url`で特定の Mod パックファイルの直リンクを取得します。
4. **Mod パックファイルのダウンロード** – 取得した URL から ZIP ファイルをダウンロードします。

Mod パックのダウンロードファイルは**ZIP 形式**で提供され、中には`manifest.json`（Mod パックの構成情報）や`mods`フォルダ（必要 Mod の一覧が入った manifest によって後からダウンロードされる）、`override`フォルダ（設定ファイル類）などが含まれま ([API Key is rate-limited of there are manual mod packs to install · Issue #2647 · itzg/docker-minecraft-server · GitHub](https://github.com/itzg/docker-minecraft-server/issues/2647#:~:text=mc_helltime%20%20,limit%20to%20reset))】。例えば manifest.json には、その Mod パックに含まれる各 Mod のプロジェクト ID やファイル ID、必要な Minecraft バージョンなどが記載されています。CLI ツールでは、この manifest を読み取り、自動で必要な Mod を一括ダウンロードするといった処理も可能です。

Rust での実装も基本的には前述の Mod ダウンロードと同じ流れです。違いは`modpackId`や`fileId`を Mod パック用のものに置き換えるだけです。例えば:

```rust
let modpack_id = 555555;  // 例: ModパックのプロジェクトID
// ... ファイル一覧を取得し、file_id（最新のModパックZIPのID）を特定 ...
let file_id = 1234567;
let url = format!("https://api.curseforge.com/v1/mods/{}/files/{}/download-url", modpack_id, file_id);
let dl_resp: Value = client
    .get(&url)
    .header("x-api-key", api_key)
    .send().unwrap()
    .json().unwrap();
let modpack_zip_url = dl_resp["data"].as_str().unwrap();
```

あとは`modpack_zip_url`からダウンロードを行い、ZIP を解凍して中の`manifest.json`を読み込むことで、その Mod パックに含まれる各 Mod（のプロジェクト ID とファイル ID）が取得できます。次に各 Mod について前述の Mod ダウンロード手順を繰り返すことで、Mod パックを構成するすべての Mod を自動インストールすることができます。

## API 制限・レートリミットに関する情報

CurseForge API を利用する上で留意すべき制限事項をまとめます。

- **ページサイズと結果上限**: `search`や`files`一覧取得系の API では 1 回のレスポンスで取得できる件数は最大 50 件で ([php-curseforge-api/openapi.yaml at master · aternosorg/php-curseforge-api · GitHub](https://github.com/aternosorg/php-curseforge-api/blob/master/openapi.yaml#:~:text=))】。また、ページネーションで取得できる総件数も最大 10,000 件までに制限されていま ([php-curseforge-api/openapi.yaml at master · aternosorg/php-curseforge-api · GitHub](https://github.com/aternosorg/php-curseforge-api/blob/master/openapi.yaml#:~:text=))】。`index`と`pageSize`を組み合わせてもうまく 10,000 件を超える範囲にはアクセスできないので、大量の結果を逐次取得する必要がある場合は、検索条件を分割するなどの工夫が必要で ([CurseForge - Archiveteam](https://wiki.archiveteam.org/index.php/CurseForge#:~:text=search%20API%20%28https%3A%2F%2Fwww,less%20than%20that%20many%20results))】。

- **レートリミット**: CurseForge API には明確な数値は公開されていませんが、一定時間あたりのリクエスト数に上限があります。例えば短時間に連続して大量のリクエストを送ると、「アクセス禁止あるいはレートリミット超過」のエラーが発生し、一時的に API 呼び出しがブロックされま ([API Key is rate-limited of there are manual mod packs to install · Issue #2647 · itzg/docker-minecraft-server · GitHub](https://github.com/itzg/docker-minecraft-server/issues/2647#:~:text=mc_helltime%20%20%7C%20%5Bmc,install%20CurseForge%20modpack))】。実際の運用では、ループで多数のファイルをダウンロードする際に短時間で制限を超えないよう**適切に間隔を空ける**、もしくは**必要なデータをまとめて取得する**（例: 複数の Mod ID を一度に指定して取得できるエンドポイントがあれば活用する）等の対策を取ってください。開発者コミュニティによれば、再試行を繰り返す自動処理などで API キーのレート制限を超過すると、しばらく待つまで 403 エラーが続くとの報告もありま ([API Key is rate-limited of there are manual mod packs to install · Issue #2647 · itzg/docker-minecraft-server · GitHub](https://github.com/itzg/docker-minecraft-server/issues/2647#:~:text=itzg%20%20%20commented%20,74))】。

- **API 利用規約**: CurseForge の API キー発行には利用目的などの審査がある背景から、取得したデータの用途についても一定のルールがあります。たとえば取得した Mod ファイルを独自に再配布することは禁止されており、あくまでユーザーの環境で自動ダウンロード・インストールを支援する目的で使用すべきです。開発時には最新の**3rd Party API 利用規約**も確認してくださ ([CurseForge 3rd Party API Terms and Conditions](https://support.curseforge.com/en/support/solutions/articles/9000207405-curse-forge-3rd-party-api-terms-and-conditions#:~:text=CurseForge%203rd%20Party%20API%20Terms,App%20exceeds%20such%20quota))】。

- **公式ドキュメント**: 詳細な API 仕様や Enum 値（例: ModLoaderType や sortField の具体的な値一覧）については、CurseForge が公開している**公式 API ドキュメント**を参照することをおすすめしま ([About the CurseForge API and How to Apply for a Key: CurseForge support](https://support.curseforge.com/en/support/solutions/articles/9000208346-about-the-curseforge-api-and-how-to-apply-for-a-key#:~:text=A%20popular%20request%20from%20day,engineered%20documentation))】。公式ドキュメントには各エンドポイントのリクエスト例やレスポンススキーマ、利用できるパラメータ値の一覧が網羅されています（例: ModLoaderType の Enum では Forge=1、Fabric=4 な ([AUTO_CURSEFORGE failing to update - new modloader type · Issue #2307 · itzg/docker-minecraft-server · GitHub](https://github.com/itzg/docker-minecraft-server/issues/2307#:~:text=String%20,data))】）。最新情報や細かな挙動の変更についても公式ドキュメントやサポートフォーラムで告知される場合があります。

以上、CurseForge API を用いた Minecraft Mod および Mod パックの検索・ダウンロード方法について解説しました。適切に API キーを管理し、API の利用制限を守りつつ、Rust から HTTP リクエストを送信することで、Mod や Mod パックの情報取得・自動インストールを安全かつ効率的に行うことができます。各種エンドポイントを組み合わせて、自作の CLI ツールに必要な機能を実装してみてください。必要に応じて引用した公式情報やドキュメントリンクも参考に、最新の API 仕様に沿った開発を進めましょう。

**参考文献・リンク:** CurseForge 公式 API ドキュメン ([Getting Started – CurseForge for Studios API](https://docs.curseforge.com/rest-api/#:~:text=Accessing%20the%20service,API%20key%20can%20be))】、CurseForge サポート記 ([About the CurseForge API and How to Apply for a Key: CurseForge support](https://support.curseforge.com/en/support/solutions/articles/9000208346-about-the-curseforge-api-and-how-to-apply-for-a-key#:~:text=How%20to%20Apply%20for%20an,API%20Key)) ([About the CurseForge API and How to Apply for a Key: CurseForge support](https://support.curseforge.com/en/support/solutions/articles/9000208346-about-the-curseforge-api-and-how-to-apply-for-a-key#:~:text=A%20popular%20request%20from%20day,engineered%20documentation))】、Aternos 社提供の API クライアント資 ([GitHub - aternosorg/php-curseforge-api: PHP Client for the CurseForge API.](https://github.com/aternosorg/php-curseforge-api#:~:text=Game%20Categories)) ([GitHub - aternosorg/php-curseforge-api: PHP Client for the CurseForge API.](https://github.com/aternosorg/php-curseforge-api#:~:text=You%20can%20fetch%20categories%20in,classId)) ([GitHub - aternosorg/php-curseforge-api: PHP Client for the CurseForge API.](https://github.com/aternosorg/php-curseforge-api#:~:text=You%20can%20search%20mods%20like,this))】など。
