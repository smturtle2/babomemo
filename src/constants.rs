use std::time::Duration;

pub const MEMO_FILE_NAME: &str = ".babomemo";
pub const CONFIG_FILE_NAME: &str = "config";
pub const PROJECT_QUALIFIER: &str = "io.github";
pub const PROJECT_ORGANIZATION: &str = "smturtle2";
pub const PROJECT_APPLICATION: &str = "babomemo";

pub const FORMAT_NOTE_MARKER: &str = "---";
pub const FORMAT_ESCAPE: char = '\\';
pub const FORMAT_ESCAPED_CHAR: char = '-';

pub const DEFAULT_HEIGHT: u16 = 5;
pub const MIN_HEIGHT: u16 = 2;
pub const MAX_HEIGHT: u16 = 40;
pub const SIZE_STEP: u16 = 1;

pub const TOOLBAR_HEIGHT: u16 = 2;
pub const STATUS_HEIGHT: u16 = 1;
pub const NOTE_GAP: u16 = 1;
pub const NOTE_BORDER_THICKNESS: u16 = 1;
pub const NOTE_BORDER_WIDTH: u16 = NOTE_BORDER_THICKNESS * 2;
pub const NOTE_BORDER_HEIGHT: u16 = NOTE_BORDER_THICKNESS * 2;
pub const NOTE_PADDING: u16 = 1;
pub const MIN_TERMINAL_WIDTH: u16 = 20;
pub const MIN_TERMINAL_HEIGHT: u16 = 6;
pub const BUTTON_HORIZONTAL_PADDING: u16 = 1;
pub const SCROLL_STEP: u32 = 3;
pub const MIN_RENDERED_NOTE_WIDTH: u16 = 3;
pub const NOTE_SEQUENCE_OFFSET: u16 = 2;
pub const NOTE_HEADER_INSET: u16 = 4;
pub const CONFIRM_WIDTH_PADDING: u16 = 6;
pub const CONFIRM_MIN_WIDTH: u16 = 32;
pub const CONFIRM_HEIGHT: u16 = 7;
pub const SETTINGS_WIDTH: u16 = 54;
pub const SETTINGS_HEIGHT: u16 = 8;
pub const MODAL_MARGIN: u16 = 2;
pub const MODAL_CONTENT_INSET: u16 = 2;
pub const MODAL_ACTION_BOTTOM_INSET: u16 = 2;
pub const SETTINGS_TITLE_ROW: u16 = 1;
pub const SETTINGS_HEIGHT_ROW: u16 = 3;
pub const SETTINGS_CONTROL_RIGHT_INSET: u16 = 9;
pub const SETTINGS_CONTROL_SPACING: u16 = 4;
pub const SETTINGS_LABEL_RIGHT_INSET: u16 = 12;
pub const BORDER_HORIZONTAL: &str = "─";
pub const BORDER_VERTICAL: &str = "│";
pub const BORDER_TOP_LEFT: &str = "┌";
pub const BORDER_TOP_RIGHT: &str = "┐";
pub const BORDER_BOTTOM_LEFT: &str = "└";
pub const BORDER_BOTTOM_RIGHT: &str = "┘";
pub const AUTOSAVE_DELAY: Duration = Duration::from_millis(300);
pub const IDLE_POLL: Duration = Duration::from_millis(100);

pub const UNDO_LIMIT: usize = 256;
pub const TAB_SPACES: &str = "    ";
