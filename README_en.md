# Vento — A Lightweight CLI Tool for File Transfer

## What is Vento?

Vento is a lightweight, fast, and easy-to-use command-line tool for file transfer and job orchestration. Inspired by HULFT, a well-known enterprise file transfer solution, Vento was born from the idea that similar automation could be achieved in a simpler, open-source CLI form.

With a single YAML configuration file, you can define flexible transfer profiles and automate file uploads/downloads between your local filesystem and remote servers over SFTP or SCP. You can also configure pre/post-transfer and error handling commands, enabling reliable and customizable batch operations.

## Key Features (v0.2.0)

- **Transfer Profiles via YAML**: Easily define source, destination, protocol, authentication, and hooks.
- **SFTP Transfers**: Upload/download files between local and SFTP servers, with SSH key authentication support.
- **SCP Transfers** (New): Supports SCP-based transfer for environments where SFTP is unavailable.
- **File Size Limitation** (New): Prevent unintended large file transfers by setting `maxFileSizeMb` in `config.yaml`. Default is 500MB. The maximum allowed value is 2048MB (2GB).
- **Validation Improvements**: Validates profile IDs and required fields in YAML, with clear error feedback.
- **Pre/Post/Error Hooks**: Execute arbitrary shell commands before/after transfers or when errors occur.
- **Simple Logging**: Output logs to file or stdout for easy debugging and tracking.

## Sample `config.yaml`

```yaml
maxFileSizeMb: 500
defaultProfileFile: "/path/to/profiles.yaml"
logLevel: "info"
logStdout: true
maxFileSizeMb: 500
```

## Sample `profiles.yaml`

```yaml
transferProfiles:
  - profileId: "daily-upload"
    description: "Upload daily reports to SFTP"
    source:
      type: "local"
      path: "/Users/youruser/reports/report.csv"
      trigger:
        type: "manual"
    destination:
      type: "sftp"
      host: "example.com"
      port: 22
      path: "/upload/reports/"
      authentication:
        method: "ssh_config"
        username: "youruser"
        sshConfigAlias: "my_server"
    transferProtocol:
      protocol: "SFTP"
    preTransferCommand: "echo 'Starting transfer...'"
    postTransferCommand: "echo 'Transfer completed.'"
    onErrorCommand: "echo 'Transfer failed.'"
```

## Getting Started

### 🧰 Install

**Option 1**: Download binaries from GitHub Releases (macOS/Linux/Windows)  
**Option 2**: Install via Homebrew  
```bash
brew tap kyotalab/vento
brew install vento
```

**Option 3**: Build from source with Rust  
```bash
git clone https://github.com/kyotalab/vento.git
cd vento
cargo build --release
```

### 🚀 Run

```bash
vento transfer --profile-id daily-upload
```

## Why Vento?

Many enterprises rely on heavyweight tools like HULFT for internal file sharing and automation. Vento aims to bring that power into the hands of developers and engineers in a lightweight, OSS-friendly CLI.

It’s designed for:
- Schedulers (cron/systemd)
- CI/CD pipelines
- Secure batch operations
- Script-based automation

## Roadmap

- HTTPS/HTTP transfers
- Plugin support
- Cross-platform packaging
- GUI wrapper (maybe!)

## Contribute

Feedback, bug reports, and PRs welcome on GitHub.  
🔗 [https://github.com/kyotalab/vento](https://github.com/kyotalab/vento)

Licensed under [MIT](./LICENSE).
