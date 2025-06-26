<div align="center">
<h1 style="display: flex; align-items: center; gap: 16px;">
  <img alt="Vento" height="160" src="logo.svg">
</h1>
</div>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-DEA584?style=for-the-badge&logo=rust&logoColor=black" alt="Rust">
  <img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT">
  <img src="https://img.shields.io/badge/Version-0.2.0-blue.svg" alt="Version: 0.2.0">
</p>


## はじめに

Vento は、シンプルで高速なファイル転送とジョブ連携を目的としたコマンドラインツールです。設定ファイル（YAML）一つで転送プロファイルを定義し、ローカルファイルシステムと SFTP サーバー間でのファイル転送を自動化できます。転送前後の処理や、異常時のジョブ実行にも対応しており、バッチ処理におけるファイル連携の自動化に貢献します。

---

## 主な機能 (v0.2.0)

* **転送プロファイルの定義**: YAML 形式の設定ファイル `profiles.yaml` で転送元、転送先、プロトコルなどを柔軟に設定できます。
* **SFTP 転送**: ローカルファイルシステムと SFTP サーバー間でのファイルのアップロード・ダウンロードに対応しています。公開鍵認証をサポートします。
* **SCP転送（新機能）**: ローカルファイルシステムをSCPサーバ間でのファイルのアップロード・ダウンロードに対応しています。公開鍵認証をサポートします。
* **転送ファイルサイズの上限指定（新機能）**: `config.yaml` に `maxFileSizeMb` を指定することで、転送ファイルサイズの最大サイズをMB単位で制限できます。デフォルトは `500MB` 、上限は `2048MB（2GB）` です。

```yaml
# ~/.config/vento/config.yaml
maxFileSizeMb: 500  # 上限は 2048MB
```

* **プロファイル設定項目のバリデーション強化（改善）**: `profile_id` や記述内容の整合性チェックを追加。設定ファイルの誤記に対して早期にエラーを通知します。
* **ジョブ連携**:
    * **転送前ジョブ**: ファイル転送の前に任意のコマンドを実行できます。
    * **転送後ジョブ**: ファイル転送が成功した後に任意のコマンドを実行できます。
    * **異常時ジョブ**: ファイル転送が失敗した場合に任意のコマンドを実行できます。
* **基本的なロギング**: 実行状況をコンソールまたはファイルに出力し、デバッグや運用をサポートします。


**⚠️ ファイルサイズバリデーションについて**
`maxFileSizeMb` で設定されたサイズを超えるファイルは、転送処理前にブロックされます。これにより意図しない大容量転送を防止します。

現在の制限上限は2GBであり、それを超える設定を行った場合は起動時にエラーが発生します。

---

## インストール

Vento は、以下の方法でインストールできます。

### 1. GitHub Releases からバイナリをダウンロード (推奨)

最も手軽な方法です。Rust 環境のセットアップは不要です。

1.  [Vento GitHub Releases ページ](https://github.com/kyotalab/vento/releases/tag/v0.1.0) にアクセスします。
2.  ご使用の OS およびアーキテクチャに対応するバイナリをダウンロードします (例: `vento-x86_64-apple-darwin.tar.gz` for macOS, `vento-x86_64-unknown-linux-gnu.tar.gz` for Linux, `vento-x86_64-pc-windows-msvc.zip` for Windows)。
3.  ダウンロードしたファイルを解凍し、実行ファイル (`vento` または `vento.exe`) をパスが通っているディレクトリ (例: `/usr/local/bin` や Windows の `C:\Windows`) に配置します。

### 2. Homebrew を使用 (macOS / Linux 推奨)

Homebrew がインストールされている場合、以下のコマンドで Vento をインストールできます。

```bash
# Vento の Homebrew Tap を追加
brew tap kyotalab/vento

# Vento をインストール
brew install vento
```

### 3. ソースコードからビルド (開発者向け)

Rust の開発環境が既にセットアップされている場合、ソースコードからビルドできます。
1. Vento リポジトリをクローンします。
```bash
git clone https://github.com/kyotalab/vento.git
cd vento
```

2. プロジェクトをビルドします。
```bash
cargo build --release
```

3. 実行ファイルは target/release/vento (Windows の場合は target/release/vento.exe) に生成されます。必要に応じてパスが通っているディレクトリに配置してください。

---

## 使い方
1. **設定ファイルの準備**
Vento は YAML 形式の設定ファイル `config.yaml` と `profiles.yaml` を使用します。

**`config.yaml` (アプリケーション全体の設定)**

デフォルトでは、ユーザーのホームディレクトリ配下の設定ディレクトリ (macOS/Linux: `~/.config/vento/config.yaml`, Windows: `%APPDATA%\vento\config.yaml`) から読み込まれます。`--config` オプションで任意のパスを指定することも可能です。

```yaml
# ~/.config/vento/config.yaml (または --config で指定したパス)
defaultProfileFile: "/path/to/your/profiles.yaml" # 転送プロファイル定義ファイルへのパス
logLevel: "info" # ログレベル: trace, debug, info, warn, error (デフォルト: info)
logFile: "/var/log/vento.log" # ログ出力先ファイル (省略時は標準出力のみ)
logStdout: false # ログをファイルにのみ出力　（デフォルト: true(ファイルと標準出力)）
```

**`profiles.yaml` (転送プロファイルの定義)**

config.yaml で指定した defaultProfileFile のパスに配置します。

```yaml
# /path/to/your/profiles.yaml
transferProfiles:
  - profileId: "daily-report-sftp"
    description: "日次レポートをSFTPサーバーにアップロードするプロファイル"
    source:
      type: "local" # or sttp
      path: "/Users/youruser/reports/daily_report.csv"
      trigger:
        type: "manual"
        # schedule: "0 0 * * * *"
    destination:
      type: "sftp" # or local
      host: "sftp.example.com"
      port: 22
      path: "/incoming/reports/daily_report.csv"
      authentication:
        method: "env_key" # or password, private_key, ssh_config
        username: "sftpuser"
        envKeyRef: "SFTP_PRIVATE_KEY_PATH" # 環境変数名。ここに秘密鍵のパスを設定
        # passwordRef: "SFTP_PASSWORD"
        # privateKeyRef: "SFTP_PRIVATE_KEY_PATH"
        # sshConfigAlias: "my_sftp_server_alias"
    transferProtocol:
      protocol: "SFTP"
    preTransferCommand: "echo '転送を開始します...' && ls -l /Users/youruser/reports/"
    postTransferCommand: "echo '転送が完了しました！' && mv /Users/youruser/reports/daily_report.csv /Users/youruser/reports/daily_report.csv.bak"
    onErrorCommand: "echo 'エラーが発生しました。管理者に連絡してください。' >> /tmp/vento_error.log"

  - profileId: "download-archive-sftp"
    description: "SFTPサーバーからアーカイブをダウンロードするプロファイル"
    source:
      type: "sftp"
      path: "/outgoing/archive.zip"
      trigger:
        type: "manual" # or schedule
        # schedule: "0 0 * * * *"
      host: "sftp.example.com"
      port: 22
      authentication:
        method: "password"
        username: "sftpuser"
        passwordRef: "SFTP_PASSWORD" # 環境変数名。ここにパスワードを設定
    destination:
      type: "local"
      path: "/Users/youruser/downloads/received_archive.zip"
    transferProtocol:
      protocol: "SFTP"
```

**⚠️ スケジュールトリガー (`trigger.type: schedule`) に関する注意点**

`trigger.type: schedule` を設定した場合、Vento は `schedule` フィールドに指定された Cron 式の**妥当性を検証**しますが、**Vento 自身がその Cron 式を解釈して定期的にプロセスを起動する機能は持っていません。**

この `schedule` 定義は、あくまで「このファイル転送プロファイルは、外部のスケジューラ（例: OS の `cron`、`systemd timer`、Windows タスクスケジューラ、Kubernetes の CronJob など）によって定期的に実行されることを意図している」という**メタデータ**として機能します。

Vento のプロセスを定期実行する場合は、お使いの環境に応じた外部のスケジューリングツールをご利用いただき、Vento コマンド (`vento transfer --profile-id <your-profile-id>`) を呼び出すように設定してください。


**⚠️ カスタムコマンドの OS 依存性に関する注意**
`preTransferCommand`, `postTransferCommand`, `onErrorCommand` に記述するコマンド文字列は、Vento が実行される OS のシェルに互換性がある必要があります。
- Linux / macOS: `sh -c "あなたのコマンド"` の形式で実行されます。Unix シェル（`ls`, `mv`, `echo`, `&&` など）の構文が利用可能です。
Windows: `cmd.exe /C "あなたのコマンド`" の形式で実行されます。Windows コマンドプロンプト（`dir`, `move`, `echo`, `&&` など）の構文が利用可能です。
異なる OS で同じプロファイルIDを使用する場合は、それぞれの OS で動作するコマンドを記述するか、OS ごとに異なる profileId を用意することを検討してください。


2. **環境変数の設定**
認証情報に `privateKeyRef` や `passwordRef` を使用する場合、対応する環境変数を設定してください。

```bash
# macOS / Linux
export SFTP_PRIVATE_KEY_PATH="/Users/youruser/.ssh/id_rsa_sftp"
export SFTP_PASSWORD="your_sftp_password"

# Windows (コマンドプロンプト)
set SFTP_PRIVATE_KEY_PATH="C:\Users\youruser\.ssh\id_rsa_sftp"
set SFTP_PASSWORD="your_sftp_password"
```

3. **転送の実行**
定義したプロファイル ID を指定して転送を実行します。

```bash
vento transfer --profile-id daily-report-sftp
```


**その他のコマンドラインオプション**

```bash
vento --help
# Usage: vento <COMMAND>

# Commands:
#   transfer  Transfer by profile in config.yaml
#   help      Print this message or the help of the given subcommand(s)

# Options:
#   -c, --config <CONFIG>  Override the default config file path
#   -h, --help             Print help
#   -V, --version          Print version

vento transfer --help
# Usage: vento transfer [OPTIONS]

# Options:
#   -p, --profile-id <PROFILE_ID>
#   -h, --help                     Print help
```

---

## ロギング
Vento は `config.yaml` の `logLevel` 設定に基づき、実行ログを出力します。
- `logLevel`: `trace`, `debug`, `info`, `warn`, `error` のいずれかを設定できます。
- `logFile`: 指定した場合、ログは標準出力だけでなく、指定されたファイルにも出力されます。

---

## エラーハンドリング
ファイル転送中にエラーが発生した場合、`onErrorCommand` で定義されたコマンドが実行されます。これにより、エラー通知やリカバリ処理などの自動化が可能です。

---

## 今後の展望
Vento はまだ初期段階のプロジェクトですが、将来的には以下の機能拡張を検討しています。
- HTTP/HTTPS などの転送プロトコル対応
- 転送スケジュールの詳細な設定（CRON形式のサポートなど）
- プラグイン機構の導入

---

## 貢献
Vento の開発にご興味をお持ちいただきありがとうございます。バグ報告、機能リクエスト、コードの改善提案など、どのような形でも貢献を歓迎します。
ライセンス
Vento は [MIT License](./LICENSE) の下で公開されています。


