<div align="center">
  <img src="assets/logo.png" width="104" alt="babomemo logo">

  <h1>babomemo</h1>

  <p><strong>A mouse-first terminal memo pad for the directory you are in.</strong></p>

  <p>
    <a href="README.ko.md">한국어</a>
    · <a href="https://github.com/smturtle2/babomemo/releases/latest">Download</a>
    · <a href="LICENSE">MIT License</a>
  </p>
</div>

<p align="center">
  <img src="assets/preview.png" width="960" alt="babomemo running in a terminal">
</p>

`babomemo` gives every directory its own lightweight memo pad. Run it where you work, write what you need, and the notes stay beside that directory in a plain-text `.babomemo` file.

No account, cloud service, or separate workspace is required.

## Why babomemo?

- **One memo file per directory.** The directory you launch from decides which notes open.
- **As many memos as you need.** Memos are stacked vertically and can be added without a limit.
- **Mouse-first editing.** Click to focus, drag to select, and use the wheel to move through the list.
- **Automatic layout.** Memos fill the terminal width, wrap their text, and grow with their content.
- **Automatic saving.** Changes are saved back to the same directory after you stop editing.
- **Terminal-native appearance.** babomemo follows your terminal's colors instead of applying its own theme.

## Installation

### Prebuilt binary (recommended)

Open the [latest release](https://github.com/smturtle2/babomemo/releases/latest) and download the archive for your system.

| System | Architecture | File |
| --- | --- | --- |
| Linux | x86-64 | `babomemo-x86_64-unknown-linux-gnu.tar.gz` |
| Windows | x86-64 | `babomemo-x86_64-pc-windows-msvc.zip` |
| macOS | Apple Silicon | `babomemo-aarch64-apple-darwin.tar.gz` |
| macOS | Intel | `babomemo-x86_64-apple-darwin.tar.gz` |

On Linux and macOS, extract the archive and place `babomemo` in a directory included in `PATH`. For example:

```sh
mkdir -p ~/.local/bin
install -m 755 babomemo ~/.local/bin/babomemo
```

Make sure `~/.local/bin` is included in `PATH`. On Windows, extract the ZIP file and place `babomemo.exe` in a directory included in `PATH`.

### Install with Cargo

If you have Rust 1.88 or newer installed:

```sh
cargo install --git https://github.com/smturtle2/babomemo --locked
```

## Quick start

Move to the directory where you want to keep notes and run `babomemo`:

```sh
cd path/to/your-project
babomemo
```

The first run creates `.babomemo` in that directory. Running `babomemo` from the same directory opens it again. Changes are saved automatically, and <kbd>Ctrl</kbd> + <kbd>D</kbd> exits after pending changes have been saved.

## Controls

### Mouse

| Action | Control |
| --- | --- |
| Focus a memo and place the cursor | Click inside the memo |
| Select text | Drag inside the memo |
| Scroll through memos | Mouse wheel |
| Add a memo | **Add memo** at the end of the list |
| Delete a memo | **Delete** on the memo border |
| Open settings | **Settings** in the upper-right corner |

### Keyboard

| Shortcut | Action |
| --- | --- |
| <kbd>Ctrl</kbd> + <kbd>N</kbd> | Add a memo |
| <kbd>Ctrl</kbd> + <kbd>D</kbd> | Save and exit |
| <kbd>Enter</kbd> | Confirm memo deletion when the confirmation is open |
| <kbd>Esc</kbd> | Save settings and close the settings window |
| <kbd>Ctrl</kbd> + <kbd>A</kbd> | Select all text in the focused memo |
| <kbd>Ctrl</kbd> + <kbd>C</kbd> / <kbd>X</kbd> / <kbd>V</kbd> | Copy / cut / paste |
| <kbd>Ctrl</kbd> + <kbd>Z</kbd> / <kbd>Y</kbd> | Undo / redo |
| <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>Z</kbd> | Redo |
| <kbd>Shift</kbd> + arrow keys | Extend the text selection |
| <kbd>Ctrl</kbd> + arrow keys | Move by word |
| <kbd>Home</kbd> / <kbd>End</kbd> | Move to the start / end of a visual line |
| <kbd>Page Up</kbd> / <kbd>Page Down</kbd> | Move through a memo by page |

Text copied with babomemo contains only the original memo text. Borders, memo numbers, buttons, padding, and visual line wrapping are not included. Terminal-native selection such as <kbd>Shift</kbd> + drag is handled by the terminal, not by babomemo.

## Storage and settings

- Notes are stored in a single UTF-8 plain-text file named `.babomemo` in the directory where the program was started. You can open the file with any text editor.
- If saving fails, babomemo shows the error and stays open instead of discarding unsaved changes on exit.
- Deleting a memo requires confirmation.
- **Settings** changes the minimum memo height. Memos still grow automatically when their content needs more rows.
- Memo width always follows the available terminal width, and text is rewrapped when the terminal is resized.
- Colors come from the terminal. The interface language follows the operating-system locale when a matching language is available.

## License

`babomemo` is available under the [MIT License](LICENSE).
