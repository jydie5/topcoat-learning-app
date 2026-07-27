# リリースと仮デプロイ

## ローカルで本番相当を確認

Topcoatは、実行ファイルとは別にアセットバンドルを必要とする。

```bash
topcoat asset bundle --release
COOKIE_SECRET="$(openssl rand -base64 48)" \
APP_ENV=production \
HOST=127.0.0.1 \
PORT=3000 \
./target/release/topcoat-learning-app
```

`target/release/topcoat-learning-app` は、祖先にある `target/assets` を自動検出する。

## コンテナ

```bash
docker build -t topcoat-notes .
docker run --rm \
  -p 3000:3000 \
  -e COOKIE_SECRET="$(openssl rand -base64 48)" \
  -v topcoat-notes-data:/data \
  topcoat-notes
```

## 今回の検証で見える運用上の注意

- SQLiteファイルを永続ボリュームへ置く必要がある。
- SQLiteを使うため、複数インスタンスの水平分割には向かない。規模が増えたら共有DBへ移行する。
- `COOKIE_SECRET` を毎回変えると署名付きCookieが無効になるため、Secret Manager等で永続管理する。
- `APP_ENV=production`ではTopcoat標準のCookie Token Store（`__Host-`、Secure、
  HttpOnly）を自動選択し、`COOKIE_SECRET`がなければ起動を拒否する。HTTPS終端が必須。
- 開発時だけ`LocalCookieTokenStore`を使い、`http://127.0.0.1`でもログインを確認できる。
- 実行ファイルだけでなく `assets` ディレクトリも同時に配布する。
- 新規DBにだけ実行する `push_schema()` は学習用。Toasty 0.7では既存テーブルに
  対して冪等ではないため、本番では履歴管理されたマイグレーションを採用する。
- Topcoat SessionのOrigin検証は有効。加えてTLS終端、バックアップ、監視、
  構造化ログ、レート制限、パスワード再設定、メール確認が別途必要。
- Topcoat 0.4は実験段階であり、バージョン更新時に破壊的変更を確認する。
- 今回、`signal` を含むソースに対する `topcoat fmt` が内部の
  `not yet implemented` で停止した。Rust標準の `cargo fmt` は利用できるが、
  Topcoat独自マクロの整形は0.4時点で不安定な箇所がある。
