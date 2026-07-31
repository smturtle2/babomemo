<div align="center">
  <img src="assets/logo.png" width="104" alt="babomemo logo">

  <h1>babomemo</h1>

  <p><strong>A mouse-first terminal memo pad that lives with your project.</strong></p>
  <p>디렉터리마다 하나씩, 필요할 때 바로 열리는 터미널 메모장.</p>

  <p>
    <a href="https://github.com/smturtle2/babomemo/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/smturtle2/babomemo/ci.yml?branch=main&style=flat-square&label=build" alt="build status"></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-7bd3b4?style=flat-square" alt="MIT license"></a>
    <img src="https://img.shields.io/badge/Rust-1.88%2B-dea584?style=flat-square&logo=rust&logoColor=white" alt="Rust 1.88 or newer">
    <img src="https://img.shields.io/badge/Linux%20%7C%20macOS%20%7C%20Windows-22262e?style=flat-square" alt="Linux, macOS and Windows">
  </p>

  <p>
    <a href="#quick-start">Quick start</a>
    · <a href="#controls">Controls</a>
    · <a href="#the-babomemo-file">File format</a>
    · <a href="#localization">Localization</a>
  </p>
</div>

<p align="center">
  <img src="assets/preview.png" width="960" alt="babomemo running in a terminal">
</p>

`babomemo`를 실행하면 현재 디렉터리의 `.babomemo`를 열고, 없으면 빈 메모 하나와 함께 새로 만듭니다. 메모는 프로젝트와 같은 자리에 남고, 별도의 워크스페이스나 계정은 필요하지 않습니다.

## Highlights

| | |
| --- | --- |
| **📁 Directory-local**<br>실행한 디렉터리의 `.babomemo`만 열고 저장합니다. | **🖱️ Mouse-first**<br>선택, 스크롤, 추가, 삭제, 설정과 종료를 마우스로 처리합니다. |
| **📝 As many memos as you need**<br>제목 없는 메모를 세로로 계속 추가하고 한 화면에서 함께 봅니다. | **↔️ Fits your terminal**<br>메모는 터미널 가로 폭을 채우고 리사이즈에 맞춰 자동으로 다시 줄바꿈됩니다. |
| **💾 Quiet, safe autosave**<br>마지막 편집 300ms 뒤 같은 디렉터리에서 원자적으로 교체 저장합니다. | **🔤 Plain text by design**<br>JSON이나 전용 DB가 아닌, 사람이 그대로 읽고 편집할 수 있는 UTF-8 파일입니다. |
| **↩️ Destructive-action undo**<br>비우기와 삭제 전 확인하고, 최근 작업부터 다시 복구할 수 있습니다. | **🌍 Locale-aware**<br>운영체제 언어를 자동 협상하며 새 Fluent 리소스를 코드 변경 없이 발견합니다. |

## Quick start

### Install from Git

Rust 1.88 이상이 필요합니다.

```sh
cargo install --git https://github.com/smturtle2/babomemo --locked
```

그다음 메모를 둘 디렉터리에서 실행합니다.

```sh
cd your-project
babomemo
```

```text
your-project/
├── src/
├── README.md
└── .babomemo    ← automatically created and saved here
```

### Prebuilt binaries

[GitHub Releases](https://github.com/smturtle2/babomemo/releases)에서 다음 타깃의 압축 파일과 `SHA256SUMS`를 제공합니다. 버전 태그가 게시되면 릴리스 워크플로가 자동으로 빌드합니다.

| Platform | Architecture |
| --- | --- |
| Linux | x86-64 |
| Windows | x86-64 |
| macOS | Apple Silicon, Intel |

## Controls

앱 명령은 마우스로, 키보드는 텍스트 편집과 탐색에만 사용합니다.

| Action | Control |
| --- | --- |
| 메모 선택·커서 배치 | 메모 본문 클릭 |
| 텍스트 선택 | 마우스 드래그 또는 <kbd>Shift</kbd> + 방향키 |
| 메모 목록 이동 | 마우스 휠 또는 오른쪽 스크롤바 |
| 메모 추가 | 상단 또는 목록 끝의 **Add memo** |
| 비우기·삭제 | 메모 테두리의 **Clear** / **Delete** |
| 전체 삭제·복구·설정·종료 | 상단 툴바 |

일반 편집 키와 함께 아래 단축키를 지원합니다.

| Shortcut | Action |
| --- | --- |
| <kbd>Ctrl</kbd> + <kbd>A</kbd> | 전체 선택 |
| <kbd>Ctrl</kbd> + <kbd>C</kbd> / <kbd>X</kbd> / <kbd>V</kbd> | 복사 / 잘라내기 / 붙여넣기 |
| <kbd>Ctrl</kbd> + <kbd>Z</kbd> / <kbd>Y</kbd> | 실행 취소 / 다시 실행 |
| <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>Z</kbd> | 다시 실행 |
| <kbd>Ctrl</kbd> + 방향키 | 단어 단위 이동 |
| <kbd>Home</kbd> / <kbd>End</kbd> / <kbd>Page Up</kbd> / <kbd>Page Down</kbd> | 줄·페이지 탐색 |

일반 마우스 드래그로 한 메모의 본문을 선택한 뒤 <kbd>Ctrl</kbd> + <kbd>C</kbd>를 누르면 원본 본문 범위만 복사됩니다. 메모 테두리, 번호, 버튼, 내부 여백과 화면 줄바꿈은 클립보드에 포함되지 않습니다. 터미널 자체 선택인 <kbd>Shift</kbd> + 드래그는 앱의 복사 경로가 아닙니다.

## The `.babomemo` file

`.babomemo`는 텍스트 편집기로 바로 열 수 있는 UTF-8 파일입니다. 새 메모를 시작하는 `---` 표식만 사용합니다.

```text
---
첫 번째 메모
---
두 번째 메모
\-\-\-
위 행의 실제 내용은 ---
C:\tmp
```

본문 행 전체가 `---`일 때만 `\-\-\-`로 기록합니다. `C:\tmp`처럼 표식과 관계없는 역슬래시는 바꾸지 않습니다.

<details>
<summary><strong>Exact format contract</strong></summary>

1. 행 전체가 정확히 `---`이면 새 메모가 시작됩니다.
2. 다음 `---` 또는 파일 끝이 현재 메모의 끝입니다. 별도의 종료 표식은 없습니다.
3. 연속된 표식과 파일의 마지막 표식은 빈 메모를 나타냅니다. 빈 파일은 메모가 0개인 상태입니다.
4. 본문의 원래 행이 `---`이면 `\-\-\-`로 직렬화합니다.
5. 본문에 이미 존재하는 `\-`는 `\\-`로 직렬화합니다.
6. 역직렬화는 구조 표식을 먼저 판정한 뒤, 본문의 `\-`만 `-`로 되돌립니다. `-`가 뒤따르지 않는 역슬래시는 그대로 유지합니다.
7. 작성기는 BOM 없는 UTF-8과 LF를 사용합니다. 읽기는 선택적인 UTF-8 BOM, LF, CRLF를 허용합니다.
8. 작성기는 버전 표식, 메모 개수, 메모 제목, 배열, 따옴표나 헤더를 기록하지 않습니다.

</details>

## Settings

상단 **Settings**에서 메모의 최소 높이를 조절합니다. 값은 운영체제의 표준 설정 디렉터리에 저장됩니다. 지원하는 설정 항목은 최소 높이뿐이며, 인식하지 못하는 항목은 무시합니다. 메모 너비는 따로 저장하지 않고 항상 터미널의 사용 가능한 가로 폭을 채웁니다.

각 메모는 테두리와 본문 사이에 상하좌우 한 칸의 여백을 둡니다. 최소 높이는 여백을 제외한 본문 행 수이며, 내용이 줄바꿈되면 본문 높이만 자동으로 늘어납니다.

전경색과 배경색도 지정하지 않습니다. 현재 터미널의 색을 그대로 사용하고, 선택·포커스·비활성 상태는 반전, 굵게, 흐리게 같은 텍스트 속성으로만 구분합니다.

메모 높이는 다음 값 중 큰 쪽을 사용합니다.

```text
max(global minimum height, wrapped content height)
```

## Localization

UI 문자열은 내장된 [Fluent](https://projectfluent.org/) 리소스에서 읽습니다. 실행 시 운영체제 로캘과 사용 가능한 BCP 47 태그를 협상하며, 최종 대체 언어는 [`locales/default`](locales/default)가 지정합니다.

새 언어를 추가할 때는 `locales/<BCP-47>.ftl`만 추가하면 됩니다. 지원 언어 목록, 언어 enum, 언어별 조건 분기는 없습니다.

```text
locales/
├── default
├── en-US.ftl
└── ko-KR.ftl
```

## Build & release

```sh
git clone https://github.com/smturtle2/babomemo.git
cd babomemo
cargo build --release --locked
```

품질 검사는 다음 명령으로 재현할 수 있습니다.

```sh
cargo fmt --check
cargo check --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
```

`v*` 태그를 푸시하면 Linux, Windows, macOS 실행 파일과 체크섬을 만드는 [release workflow](.github/workflows/release.yml)가 실행됩니다.

## License

`babomemo` is available under the [MIT License](LICENSE).
