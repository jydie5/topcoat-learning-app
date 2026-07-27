# Topcoat Notes — Rustフルスタック学習用アプリ

[![CI](https://github.com/jydie5/topcoat-learning-app/actions/workflows/ci.yml/badge.svg)](https://github.com/jydie5/topcoat-learning-app/actions/workflows/ci.yml)

Rustの実験的フルスタックWebフレームワーク「Topcoat 0.4」を、メモ／ToDoアプリを作りながら学ぶためのリポジトリです。

> [!WARNING]
> 学習・検証用です。Topcoat 0.4は初期段階かつ実験的で、破壊的変更が予告されています。このアプリをそのまま本番サービスへ利用しないでください。

調査日: 2026-07-24
対象: Rust のフルスタック Web フレームワーク [`tokio-rs/topcoat`](https://github.com/tokio-rs/topcoat)

## このリポジトリで動かせるもの

調査内容を実際に追える、メモ／ToDoアプリを実装済み。

- `/signup`、`/login`、`/logout`: Topcoat Sessionによる認証
- `/todos`: ユーザー別一覧、追加フォーム、2方式のリアルタイム検索
- `/todos/{todo_id}`: 詳細、`#[procedure]` インライン編集、完了切り替え、削除
- SQLiteへの永続化（Toasty）
- `signal` による追加フォームの開閉
- `#[shard]` によるサーバー側検索と部分更新
- htmxによる同じ検索の部分更新（実装差を横並び比較）
- `#[memoize]` によるリクエスト内のユーザー／集計取得共有
- Tailwindと、プロジェクトへ取り込んだTopcoat UI
- カスタムindigoテーマとダークモード
- 404画面、フォーム入力検証、所有権チェック
- 署名付きCookieによるブラウザ別訪問回数
- 5秒TTLの集計キャッシュと、書き込み時の明示的な無効化
- リリース用アセットバンドルとコンテナ例

### 起動

RustとTopcoat CLIを導入後、プロジェクトルートで実行する。

```bash
cargo install topcoat-cli --version 0.4.0 --locked
topcoat dev
```

ブラウザで `http://127.0.0.1:3000` を開く。
初回は `/signup` で自分の学習用アカウントを作成する。データベースやサンプルアカウントはリポジトリに含めていない。

テストとビルド:

```bash
cargo test
cargo check
topcoat asset bundle --release
```

### 学習用の構成

```text
topcoat/
├── src/
│   ├── main.rs                    # 起動、DB、依存関係の組み立て
│   ├── app.rs                     # ルーター、共通レイアウト、/
│   ├── app/
│   │   ├── login.rs               # Sessionログイン
│   │   ├── signup.rs              # Argon2でユーザー登録
│   │   ├── logout.rs              # Session破棄
│   │   ├── htmx_search.rs         # htmx用HTML断片
│   │   ├── todos.rs               # GET/POST /todos、signal、shard、memoize
│   │   └── todos/
│   │       ├── id.rs              # 詳細とprocedure編集
│   │       └── id/
│   │           ├── toggle.rs      # POST .../toggle
│   │           └── delete.rs      # POST .../delete
│   ├── auth.rs                    # memoizeしたcurrent_user
│   ├── sessions.rs                # ローカルHTTP用TokenStore
│   ├── validation.rs              # サーバー側入力検証
│   ├── domain/                    # User / SessionRecord / Todo
│   ├── repository/                # SQLite/Toastyだけを知るDB層
│   ├── services/                  # ユースケースとキャッシュ制御
│   ├── cache/                     # プロセス内TTLキャッシュ
│   ├── cookies.rs                 # 署名付きCookie
│   └── components/                # Topcoat UIから取り込んだ編集可能な部品
├── build.rs                       # Tailwindとアイコンのビルド
├── styles.css                     # Tailwind入力とテーマトークン
├── components.toml                # Topcoat UIの導入状態
├── .github/                        # CIとDependabot設定
├── Dockerfile
├── SECURITY.md
└── DEPLOYMENT.md
```

リクエストの流れ:

```text
Browser
  ↓ HTTP / shard / procedure / htmx
src/app/*（ルートと画面）
  ↓ Session + #[memoize]
AuthService / TodoService
  ├─→ TodoStatsCache
  └─→ TodoRepository
          ↓
      Toasty + SQLite（user_idで分離）
```

画面からDBを直接触らず、`app → service → repository → database` の一方向にしている。キャッシュの利用と無効化はサービス層へ集約したため、各層を順に読めば責務を追える。

本番相当の起動方法と運用上の不足は [`DEPLOYMENT.md`](./DEPLOYMENT.md) を参照。

### セキュリティ設計

- パスワードはArgon2でハッシュ化し、平文では保存しない。
- セッションの生トークンはブラウザだけが保持し、DBにはSHA-256ハッシュを保存する。
- Todoの参照・更新・削除はすべてログインユーザーの`user_id`で絞り込む。
- Topcoat SessionのOrigin検証と、サーバー側の入力検証を有効にする。
- `APP_ENV=production`ではSecure・HttpOnlyなTopcoat標準Cookieを使い、`COOKIE_SECRET`がなければ起動を拒否する。
- ローカルDB、バックアップ、`.env`はGitとDockerの対象外にする。
- CDNから読むhtmxにはバージョン固定とSubresource Integrity（SRI）を設定する。

公開前に`cargo audit`、テスト、ビルド、秘密情報スキャンを実施している。脆弱性の連絡方法は [`SECURITY.md`](./SECURITY.md) を参照。

### 既知の制約

- レート制限、メール確認、パスワード再設定、失効セッションの定期削除、CSP等の本番向け機能は未実装。
- Topcoat 0.4では`procedure`完了後に同じ画面の`signal`を更新するとブラウザが応答しなくなるケースがあったため、インライン編集は保存処理だけを`#[procedure]`で行う。保存結果は一覧へ戻って確認する。
- `push_schema()`は新規SQLite DB向け。既存環境のスキーマ変更には履歴管理されたマイグレーションが必要。

> [!NOTE]
> `topcoat.io` にある同名の古い CSS フレームワークとは別物。本書で扱うのは Tokio プロジェクト配下の Rust フレームワーク。

## ひとことで

Topcoat は、Rust だけでサーバー描画・ルーティング・HTML テンプレート・ブラウザ上のリアクティブ処理・アセット管理までまとめて扱おうとする、モジュール式の「batteries-included」フルスタック Web フレームワーク。

思想は「Rust で普通のサーバーコードを書きながら、別API・別フロントエンド・Wasmビルドの定型作業を減らす」。公式は “The full full-stack framework for Rust” と表現している。

ただし、現時点では**初期段階かつ実験的**。公式も破壊的変更を予告しており、本番採用より、まず小さな検証アプリで感触を確かめる段階にある。

## 現在の基本情報

| 項目 | 内容 |
| --- | --- |
| 最新ドキュメント版 | `0.4.0`（2026-07-22公開） |
| リポジトリ | [`tokio-rs/topcoat`](https://github.com/tokio-rs/topcoat) |
| APIドキュメント | [`docs.rs/topcoat`](https://docs.rs/topcoat/latest/topcoat/) |
| ライセンス | MIT |
| 実行基盤 | Tokio |
| 開発状態 | Early-stage / experimental |

## 仕組みの全体像

1. リクエストを Topcoat の `Router` が受け取る。
2. `#[page]`、`#[layout]`、APIルートなどへ振り分ける。
3. `view!` マクロと `#[component]` でHTMLをサーバー描画する。
4. ブラウザだけで完結する状態更新は、Rustで書いた `$(...)` 式をJavaScriptに変換して実行する。
5. サーバー処理が必要な部分更新は `#[shard]` を再描画し、返されたHTMLで該当箇所を差し替える。

つまり、基本はSSRで、必要な対話性だけをクライアントまたはサーバー部分更新として加える構成。独立したJSON APIとSPAを最初から二重に作る設計ではない。

## 主な特徴

### 1. Rust中心のSSR

- すべてのマークアップをサーバーで描画する。
- コンポーネントは `async fn` にでき、サーバー側から直接DB等へアクセスできる。
- フロントエンド用API層を別に設ける定型作業を減らせる。

### 2. Wasmなしのクライアント・リアクティビティ

- `view!` 内に `signal` で状態を書く。
- `$(...)` 内は型チェックされるRust式。
- 初回はサーバーで評価し、ブラウザ用にはJavaScriptへ変換する。
- 公式説明では、Wasmバンドルもクライアント用ビルド工程も不要。

対応範囲は独自の「Rust/JavaScript両用の式言語」であり、任意のRustコードがそのままブラウザで動く、という意味ではない点に注意。

```rust
view! {
    signal open = false;

    <button @click=$(|_e| open.set(!open.get()))>
        "開閉"
    </button>
    <p :hidden=$(!open.get())>
        "ブラウザ上で即座に更新される"
    </p>
}
```

### 3. サーバー側の部分再描画

`#[shard]` を付けたコンポーネントは、リアクティブな引数が変化するとサーバーで再描画され、該当HTMLだけが差し替えられる。検索候補やDB依存の一覧などに向く。

ブラウザから呼べる非同期サーバー関数として `procedure` も用意されている。

### 4. HTMLに近い `view!` テンプレート

- HTML風の構文内で、Rustの `for` や `if` を使える。
- `#[component]` で非同期関数をUIコンポーネント化できる。
- `attributes!` や `class!` で属性・クラスを組み立てられる。
- `topcoat fmt` で `view!` 等のマクロ内部も整形できる。

### 5. モジュール構造ベースのルーティング

手動ルート登録に加え、Rustのモジュール／ファイル構造からルートを自動発見できる。

```text
src/
├── app.rs                  → /
└── app/
    ├── about.rs            → /about
    ├── _marketing.rs       → URLを増やさないレイアウト
    ├── _marketing/
    │   └── pricing.rs      → /pricing
    ├── posts.rs            → /posts
    ├── posts/
    │   └── id.rs           → /posts/{post_id}
    └── api/
        └── health.rs       → GET /api/health
```

### 6. 編集可能なUIコンポーネント

Topcoat UI は Tailwind ベースで、shadcn/ui に着想を得た方式。CLIでコンポーネントをプロジェクト内へコピーし、利用側が自由に編集する。

### 7. アセットとTailwindの統合

- `asset!` 宣言をコンパイル済みバイナリから検出してバンドルする。
- コンテンツハッシュ付きURLと強いブラウザキャッシュを使える。
- Webフォント、Iconifyのアイコンに対応する。
- TailwindをNodeなしでアセットパイプラインへ統合できる。
- htmx、Alpine AJAX向けの統合もある。

### 8. リクエスト周辺

- `Cx` によるリクエストコンテキスト
- 型をキーにしたアプリ共有コンテキスト
- `#[memoize]` によるリクエスト単位のキャッシュと重複処理抑制
- Cookie（署名・暗号化・prefix対応）
- ストレージを利用者が選べるセッション機構

## 最小構成で動かす

公式のGetting Startedは次の流れ。

```bash
cargo new hello-world
cd hello-world
cargo add topcoat
cargo add tokio --features rt-multi-thread,macros
```

`src/main.rs`:

```rust
use topcoat::{
    Result,
    router::{Router, RouterBuilderDiscoverExt, page},
    view::{component, view},
};

#[tokio::main]
async fn main() {
    topcoat::start(Router::builder().discover().build())
        .await
        .unwrap();
}

#[page("/")]
async fn home() -> Result {
    view! {
        <!DOCTYPE html>
        <html>
            <head>
                <title>"Hello world"</title>
                topcoat::dev::script()
            </head>
            <body>
                hello(name: "World")
            </body>
        </html>
    }
}

#[component]
async fn hello(name: &str) -> Result {
    view! {
        <h1>"Hello, " (name) "!"</h1>
    }
}
```

通常の起動:

```bash
cargo run
```

標準では `http://127.0.0.1:3000`。

日常の開発にはCLIを使う。

```bash
cargo install topcoat-cli
topcoat dev
```

`topcoat dev` はビルド、アセットのバンドル、サーバー起動、変更監視、再起動をまとめて行う。ページに `topcoat::dev::script()` があれば再ビルド後に自動リロードされる。

## 現時点の注意点

### 採用上の大きな注意

- 公式が実験的と明記しており、APIの破壊的変更が見込まれる。
- ドキュメントにない挙動や、エコシステムの薄さに自力で対応する可能性がある。
- Rust式からJavaScriptへの変換は魅力的だが、新しい独自モデルなので、対応構文・デバッグ・ブラウザ互換性を実アプリで検証したい。
- 一般的なReact等の資産をそのまま使う構成とは異なる。
- デプロイ手順そのものが公式ロードマップ上にあり、現状は運用面の情報がまだ成熟していない。

### まだロードマップにあるもの

以下は「今後予定」であり、現時点で完成済みとみなさない方がよい。

- `topcoat new` によるプロジェクト雛形生成
- 静的出力、静的ページの事前描画
- リアクティビティの拡充
- バリデーション、OpenAPI
- デプロイ文書
- Streaming SSR / Suspense
- クライアントサイドナビゲーションとprefetch
- WebSocket、WebTransport、SSE
- 画像最適化
- 認証、バックグラウンドジョブ
- Islands

なお、基本的なセッション認証の機械部分はすでに存在する一方、より包括的な「認証」はロードマップに残っている。両者は分けて考える必要がある。

## どんな用途に合いそうか

現時点で試しやすい:

- Rustだけで小さな管理画面や業務ツールを作る検証
- SSR主体で、フォームや部分更新が中心のアプリ
- APIとSPAを別々に保守したくない小規模プロジェクト
- 新しいRust Web開発モデルを評価するプロトタイプ

慎重にしたい:

- 長期安定性が最優先の本番システム
- 複雑なクライアント状態や大規模SPA
- 成熟した認証、運用、デプロイ手順が最初から必須
- React/Vue等の既存コンポーネント資産を大量に再利用したい案件

## 最初の検証案

まずは「メモ／ToDoアプリ」が適している。

1. 一覧と詳細ページでルーティングを確認
2. 入力フォームとDB保存でサーバー処理を確認
3. 開閉UIでクライアント側 `signal` を確認
4. 検索フィルターを `#[shard]` で部分更新
5. TailwindとTopcoat UIを導入
6. Cookieまたはセッションを試す
7. リリースビルドと仮デプロイで運用上の不足を洗い出す

この一巡で、Topcoat独自の強みと、現段階で許容できる未成熟さをかなり判断できる。

## 公式情報

- [公式GitHubリポジトリ](https://github.com/tokio-rs/topcoat)
- [公式APIドキュメント（docs.rs）](https://docs.rs/topcoat/latest/topcoat/)
- [公式Getting Started](https://github.com/tokio-rs/topcoat/blob/main/crates/topcoat/docs/getting_started.md)
- [crates.io: topcoat](https://crates.io/crates/topcoat)
- [crates.io: topcoat-cli](https://crates.io/crates/topcoat-cli)
