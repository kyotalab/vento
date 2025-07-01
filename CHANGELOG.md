# Changelog

Please see [GitHub Releases](https://github.com/kyotalab/vento/releases) for a complete update history.

## [v0.3.0] - 2025-07-01

### Added
- `vento admin` TUI interface for managing transfer profiles
- Support for:
    - Viewing profiles in list view
    - Editing profiles interactively
    - Creating new profiles (Ctrl+N)
    - Deleting profiles (Ctrl+D)
    - Duplicating profiles (Ctrl+C)
    - Config editing (Tab to switch)
- Cursor movement and editing enhancements (arrow keys, cursor position shown)

### Changed
- Keyboard shortcuts updated for improved usability (Tab, Shift+Tab, Ctrl+S, etc.)

### Fixed
- Profile and config YAML changes now persist correctly on save

---

## [v0.2.0] - 2025-06-26

### Added
- **SCP protocol support**
    - Now possible to transfer files between local and remote using SCP (supports both upload and download).
- **Added file size upper limit validation**
`maxFileSizeMb` can now be specified in `config.yaml`. Default is 500MB. The upper limit is limited to 2GB (2048MB).
- **Strengthened validation of profile definition**
Added character count and format constraints at the structure level, such as `profileId`, `description`, `command`, and `path`. Prevents configuration errors before execution.

### Improvements and fixes
- Changed file transfer processing to stream transfer using `BufReader`, allowing stable support for large files (several hundred MB to GB).
- Refactored the entire code and improved logging.

### Compatibility Notes
- You may need to modify your configuration file by adding a new `maxFileSizeMb` to `config.yaml` (if not specified, it will default to 500MB).

---

## [v0.1.0] - 2025-06-01
- Initial release (file transfer via SFTP, job linking function, etc.)

---

