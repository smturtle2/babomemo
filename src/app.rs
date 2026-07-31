use std::{
    path::{Path, PathBuf},
    time::Instant,
};

use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use fluent_bundle::FluentArgs;
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Position, Rect},
    style::Style,
    widgets::{Block, Widget},
};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    clipboard::SystemClipboard,
    config::Config,
    constants::{
        AUTOSAVE_DELAY, BORDER_BOTTOM_LEFT, BORDER_BOTTOM_RIGHT, BORDER_HORIZONTAL,
        BORDER_TOP_LEFT, BORDER_TOP_RIGHT, BORDER_VERTICAL, BUTTON_HORIZONTAL_PADDING,
        CONFIRM_HEIGHT, CONFIRM_MIN_WIDTH, CONFIRM_WIDTH_PADDING, DELETION_UNDO_LIMIT, MAX_HEIGHT,
        MIN_HEIGHT, MIN_RENDERED_NOTE_WIDTH, MIN_TERMINAL_HEIGHT, MIN_TERMINAL_WIDTH,
        MODAL_ACTION_BOTTOM_INSET, MODAL_CONTENT_INSET, MODAL_MARGIN, NOTE_BORDER_HEIGHT,
        NOTE_BORDER_THICKNESS, NOTE_BORDER_WIDTH, NOTE_GAP, NOTE_HEADER_INSET, NOTE_PADDING,
        NOTE_SEQUENCE_OFFSET, SCROLL_STEP, SCROLLBAR_THUMB, SCROLLBAR_TRACK, SCROLLBAR_WIDTH,
        SETTINGS_CONTROL_RIGHT_INSET, SETTINGS_CONTROL_SPACING, SETTINGS_HEIGHT,
        SETTINGS_HEIGHT_ROW, SETTINGS_LABEL_RIGHT_INSET, SETTINGS_TITLE_ROW, SETTINGS_WIDTH,
        SIZE_STEP, STATUS_HEIGHT, TAB_SPACES, TOOLBAR_HEIGHT,
    },
    document::Document,
    editor::{Editor, VisualRow, cursor_row_column, display_grapheme, display_width, visual_rows},
    i18n::I18n,
    storage,
    styles::TerminalStyle,
};

#[derive(Clone)]
enum Action {
    Add,
    Settings,
    Restore,
    DeleteAll,
    Quit,
    Clear(usize),
    Delete(usize),
    Confirm,
    Cancel,
    SettingsHeightLess,
    SettingsHeightMore,
    SettingsDone,
}

enum Pending {
    Clear(usize),
    Delete(usize),
    DeleteAll,
}

enum Deleted {
    Clear { index: usize, text: String },
    Note { index: usize, text: String },
    All { notes: Vec<String> },
}

enum Drag {
    Text { note: usize, anchor: usize },
    Scrollbar,
}

struct Target {
    area: Rect,
    action: Action,
}

#[derive(Clone, Copy)]
struct NoteLayout {
    top: u32,
    x: u16,
    width: u16,
    text_width: u16,
    text_height: u16,
}

impl NoteLayout {
    fn text_x(self) -> u16 {
        self.x.saturating_add(NOTE_BORDER_THICKNESS + NOTE_PADDING)
    }

    fn text_top(self) -> u32 {
        self.top
            .saturating_add(u32::from(NOTE_BORDER_THICKNESS + NOTE_PADDING))
    }

    fn text_bottom(self) -> u32 {
        self.text_top().saturating_add(u32::from(self.text_height))
    }

    fn bottom_border(self) -> u32 {
        self.text_bottom().saturating_add(u32::from(NOTE_PADDING))
    }

    fn height(self) -> u32 {
        u32::from(
            self.text_height
                .saturating_add(NOTE_BORDER_HEIGHT)
                .saturating_add(NOTE_PADDING.saturating_mul(2)),
        )
    }
}

pub struct App {
    path: PathBuf,
    editors: Vec<Editor>,
    clipboard: SystemClipboard,
    config: Config,
    i18n: I18n,
    focused: Option<usize>,
    scroll: u32,
    total_height: u32,
    viewport: Rect,
    layouts: Vec<NoteLayout>,
    targets: Vec<Target>,
    drag: Option<Drag>,
    pending: Option<Pending>,
    settings_open: bool,
    deleted: Vec<Deleted>,
    dirty_since: Option<Instant>,
    save_error: Option<String>,
    transient_error: Option<String>,
    quit: bool,
}

impl App {
    pub fn new(
        path: PathBuf,
        document: Document,
        config: Config,
        i18n: I18n,
        config_error: Option<String>,
    ) -> Self {
        let editors = document.notes.into_iter().map(Editor::new).collect();
        Self {
            path,
            editors,
            clipboard: SystemClipboard::new(),
            config,
            i18n,
            focused: None,
            scroll: 0,
            total_height: 0,
            viewport: Rect::default(),
            layouts: Vec::new(),
            targets: Vec::new(),
            drag: None,
            pending: None,
            settings_open: false,
            deleted: Vec::new(),
            dirty_since: None,
            save_error: None,
            transient_error: config_error,
            quit: false,
        }
    }

    pub fn should_quit(&self) -> bool {
        self.quit
    }

    pub fn poll_timeout(&self) -> std::time::Duration {
        self.dirty_since
            .map(|since| AUTOSAVE_DELAY.saturating_sub(since.elapsed()))
            .unwrap_or(crate::constants::IDLE_POLL)
            .min(crate::constants::IDLE_POLL)
    }

    pub fn autosave_if_due(&mut self) {
        if self
            .dirty_since
            .is_some_and(|since| since.elapsed() >= AUTOSAVE_DELAY)
        {
            self.save();
        }
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key) => self.handle_key(key),
            Event::Mouse(mouse) => self.handle_mouse(mouse),
            Event::Paste(text) => self.insert_text(&normalize_paste(&text)),
            Event::Resize(_, _) | Event::FocusGained | Event::FocusLost => {}
        }
    }

    pub fn render(&mut self, frame: &mut Frame<'_>) {
        let area = frame.area();
        self.targets.clear();
        let mut cursor = None;
        {
            let buffer = frame.buffer_mut();
            Block::default()
                .style(TerminalStyle::background())
                .render(area, buffer);
            if area.width < MIN_TERMINAL_WIDTH || area.height < MIN_TERMINAL_HEIGHT {
                self.render_too_small(buffer, area);
            } else {
                self.render_toolbar(buffer, area);
                self.viewport = Rect::new(
                    area.x,
                    area.y + TOOLBAR_HEIGHT,
                    area.width,
                    area.height - TOOLBAR_HEIGHT - STATUS_HEIGHT,
                );
                cursor = self.render_notes(buffer);
                self.render_status(buffer, area);
                if self.settings_open {
                    cursor = None;
                    self.render_settings(buffer, area);
                } else if self.pending.is_some() {
                    cursor = None;
                    self.render_confirmation(buffer, area);
                }
            }
        }
        if let Some(position) = cursor {
            frame.set_cursor_position(position);
        }
    }

    fn render_too_small(&mut self, buffer: &mut Buffer, area: Rect) {
        put_text(
            buffer,
            area.x,
            area.y,
            &self.i18n.text("status-too-small"),
            area.width,
            TerminalStyle::error(),
        );
        let label = self.i18n.text("button-quit");
        let width = button_width(&label).min(area.width);
        let x = area.right().saturating_sub(width);
        if let Some(rect) = draw_button(
            buffer,
            x,
            area.bottom().saturating_sub(1),
            area.right(),
            &label,
            TerminalStyle::button(),
        ) {
            self.targets.push(Target {
                area: rect,
                action: Action::Quit,
            });
        }
    }

    fn render_toolbar(&mut self, buffer: &mut Buffer, area: Rect) {
        buffer.set_style(
            Rect::new(area.x, area.y, area.width, TOOLBAR_HEIGHT),
            TerminalStyle::toolbar(),
        );

        let quit_label = self.i18n.text("button-quit");
        let quit_width = button_width(&quit_label).min(area.width);
        let quit_x = area.right().saturating_sub(quit_width);
        if let Some(rect) = draw_button(
            buffer,
            quit_x,
            area.y,
            area.right(),
            &quit_label,
            TerminalStyle::button(),
        ) {
            self.targets.push(Target {
                area: rect,
                action: Action::Quit,
            });
        }

        let items = [
            ("button-add", Action::Add, true, false),
            ("button-settings", Action::Settings, true, false),
            (
                "button-restore",
                Action::Restore,
                !self.deleted.is_empty(),
                false,
            ),
            (
                "button-delete-all",
                Action::DeleteAll,
                !self.editors.is_empty(),
                true,
            ),
        ];
        let mut x = area.x;
        let mut y = area.y;
        for (id, action, enabled, destructive) in items {
            let label = self.i18n.text(id);
            let width = button_width(&label);
            let row_right = if y == area.y { quit_x } else { area.right() };
            if x.saturating_add(width) > row_right {
                y = y.saturating_add(1);
                x = area.x;
            }
            if y >= area.y + TOOLBAR_HEIGHT || x.saturating_add(width) > area.right() {
                continue;
            }
            let style = if !enabled {
                TerminalStyle::disabled()
            } else if destructive {
                TerminalStyle::destructive_button()
            } else {
                TerminalStyle::button()
            };
            if let Some(rect) = draw_button(buffer, x, y, area.right(), &label, style) {
                if enabled {
                    self.targets.push(Target { area: rect, action });
                }
                x = rect.right().saturating_add(1);
            }
        }
    }

    fn render_notes(&mut self, buffer: &mut Buffer) -> Option<Position> {
        let usable_width = self.viewport.width.saturating_sub(SCROLLBAR_WIDTH).max(1);
        let outer_width = usable_width;
        let text_width = outer_width
            .saturating_sub(NOTE_BORDER_WIDTH + NOTE_PADDING.saturating_mul(2))
            .max(1);
        let x = self.viewport.x;

        self.layouts.clear();
        let mut top = 0_u32;
        for editor in &self.editors {
            let row_count = visual_rows(editor.text(), text_width)
                .len()
                .min(u16::MAX as usize) as u16;
            let text_height = self.config.default_height.max(row_count);
            let layout = NoteLayout {
                top,
                x,
                width: outer_width,
                text_width,
                text_height,
            };
            self.layouts.push(layout);
            top = top
                .saturating_add(layout.height())
                .saturating_add(u32::from(NOTE_GAP));
        }
        let add_top = top;
        self.total_height = add_top.saturating_add(1);
        self.clamp_scroll();

        let mut cursor = None;
        for index in 0..self.editors.len() {
            let layout = self.layouts[index];
            if !self.layout_visible(layout) {
                continue;
            }
            if let Some(position) = self.render_note(buffer, index, layout) {
                cursor = Some(position);
            }
        }

        if self.editors.is_empty() {
            put_text(
                buffer,
                self.viewport.x,
                self.viewport.y,
                &self.i18n.text("status-empty"),
                usable_width,
                TerminalStyle::disabled(),
            );
        }
        if let Some(y) = self.screen_y(add_top) {
            let label = self.i18n.text("button-add");
            let width = button_width(&label).min(usable_width);
            let add_x = self.viewport.x + usable_width.saturating_sub(width) / 2;
            if let Some(rect) = draw_button(
                buffer,
                add_x,
                y,
                self.viewport.x + usable_width,
                &label,
                TerminalStyle::button(),
            ) {
                self.targets.push(Target {
                    area: rect,
                    action: Action::Add,
                });
            }
        }
        self.render_scrollbar(buffer);
        cursor
    }

    fn render_note(
        &mut self,
        buffer: &mut Buffer,
        index: usize,
        layout: NoteLayout,
    ) -> Option<Position> {
        let focused = self.focused == Some(index);
        let border_style = TerminalStyle::border(focused);
        let bottom = layout.bottom_border();

        if let Some(y) = self.screen_y(layout.top) {
            draw_horizontal_border(buffer, layout.x, y, layout.width, true, border_style);
            let sequence = format!(" {} ", index + 1);
            put_text(
                buffer,
                layout.x.saturating_add(NOTE_SEQUENCE_OFFSET),
                y,
                &sequence,
                layout.width.saturating_sub(NOTE_HEADER_INSET),
                border_style,
            );
            self.render_note_buttons(buffer, index, layout, y);
        }
        if let Some(y) = self.screen_y(bottom) {
            draw_horizontal_border(buffer, layout.x, y, layout.width, false, border_style);
        }
        let side_height = layout
            .height()
            .saturating_sub(u32::from(NOTE_BORDER_HEIGHT));
        for row in 0..side_height {
            let absolute = layout
                .top
                .saturating_add(u32::from(NOTE_BORDER_THICKNESS))
                .saturating_add(row);
            if let Some(y) = self.screen_y(absolute) {
                put_cell(buffer, layout.x, y, BORDER_VERTICAL, border_style);
                put_cell(
                    buffer,
                    layout.x + layout.width.saturating_sub(1),
                    y,
                    BORDER_VERTICAL,
                    border_style,
                );
            }
        }

        let editor = &self.editors[index];
        let rows = visual_rows(editor.text(), layout.text_width);
        let selection = editor.selection();
        for (row_index, row) in rows.iter().enumerate() {
            if row_index >= usize::from(layout.text_height) {
                break;
            }
            let absolute = layout.text_top().saturating_add(row_index as u32);
            let Some(y) = self.screen_y(absolute) else {
                continue;
            };
            render_text_row(
                buffer,
                layout.text_x(),
                y,
                layout.text_width,
                editor.text(),
                row,
                selection.as_ref(),
            );
        }

        if focused {
            let (row, column) = cursor_row_column(editor.text(), editor.cursor(), &rows);
            let absolute = layout.text_top().saturating_add(row as u32);
            self.screen_y(absolute).map(|y| {
                Position::new(
                    layout
                        .text_x()
                        .saturating_add(column.min(layout.text_width.saturating_sub(1))),
                    y,
                )
            })
        } else {
            None
        }
    }

    fn render_note_buttons(
        &mut self,
        buffer: &mut Buffer,
        index: usize,
        layout: NoteLayout,
        y: u16,
    ) {
        let clear = self.i18n.text("button-clear");
        let delete = self.i18n.text("button-delete");
        let clear_width = button_width(&clear);
        let delete_width = button_width(&delete);
        let right = layout.x + layout.width.saturating_sub(1);
        let delete_x = right.saturating_sub(delete_width);
        let clear_x = delete_x.saturating_sub(1 + clear_width);
        if clear_x <= layout.x.saturating_add(NOTE_HEADER_INSET) {
            return;
        }
        if let Some(rect) = draw_button(
            buffer,
            clear_x,
            y,
            delete_x.saturating_sub(1),
            &clear,
            TerminalStyle::destructive_button(),
        ) {
            self.targets.push(Target {
                area: rect,
                action: Action::Clear(index),
            });
        }
        if let Some(rect) = draw_button(
            buffer,
            delete_x,
            y,
            right,
            &delete,
            TerminalStyle::destructive_button(),
        ) {
            self.targets.push(Target {
                area: rect,
                action: Action::Delete(index),
            });
        }
    }

    fn render_scrollbar(&self, buffer: &mut Buffer) {
        if self.viewport.height == 0 {
            return;
        }
        let x = self.viewport.right().saturating_sub(1);
        for y in self.viewport.y..self.viewport.bottom() {
            put_cell(buffer, x, y, SCROLLBAR_TRACK, TerminalStyle::scrollbar());
        }
        if self.total_height <= u32::from(self.viewport.height) {
            return;
        }
        let height = u32::from(self.viewport.height);
        let thumb_height = ((height * height) / self.total_height).max(1).min(height);
        let travel = height.saturating_sub(thumb_height);
        let max_scroll = self.max_scroll();
        let thumb_start = self
            .scroll
            .saturating_mul(travel)
            .checked_div(max_scroll)
            .unwrap_or(0);
        for offset in 0..thumb_height {
            put_cell(
                buffer,
                x,
                self.viewport.y + (thumb_start + offset) as u16,
                SCROLLBAR_THUMB,
                TerminalStyle::scrollbar_thumb(),
            );
        }
    }

    fn render_status(&self, buffer: &mut Buffer, area: Rect) {
        let y = area.bottom().saturating_sub(1);
        let rect = Rect::new(area.x, y, area.width, STATUS_HEIGHT);
        buffer.set_style(rect, TerminalStyle::status());
        let (message, style) = if let Some(error) = &self.save_error {
            let mut args = FluentArgs::new();
            args.set("error", error.as_str());
            (
                self.i18n.format("status-save-error", Some(&args)),
                TerminalStyle::error(),
            )
        } else if let Some(error) = &self.transient_error {
            (error.clone(), TerminalStyle::error())
        } else if self.dirty_since.is_some() {
            (self.i18n.text("status-saving"), TerminalStyle::status())
        } else {
            let mut args = FluentArgs::new();
            args.set("file", file_label(&self.path));
            args.set("count", self.editors.len() as i64);
            (
                self.i18n.format("status-saved", Some(&args)),
                TerminalStyle::status(),
            )
        };
        put_text(buffer, area.x, y, &message, area.width, style);
    }

    fn render_confirmation(&mut self, buffer: &mut Buffer, area: Rect) {
        let mut args = FluentArgs::new();
        let Some(pending) = self.pending.as_ref() else {
            return;
        };
        let id = match pending {
            Pending::Clear(index) => {
                args.set("number", (*index + 1) as i64);
                "confirm-clear"
            }
            Pending::Delete(index) => {
                args.set("number", (*index + 1) as i64);
                "confirm-delete"
            }
            Pending::DeleteAll => "confirm-delete-all",
        };
        let message = self.i18n.format(id, Some(&args));
        let modal = centered(
            area,
            display_width(&message)
                .saturating_add(CONFIRM_WIDTH_PADDING)
                .max(CONFIRM_MIN_WIDTH),
            CONFIRM_HEIGHT,
        );
        draw_box(buffer, modal, TerminalStyle::modal());
        put_text(
            buffer,
            modal.x.saturating_add(MODAL_CONTENT_INSET),
            modal.y.saturating_add(MODAL_CONTENT_INSET),
            &message,
            modal
                .width
                .saturating_sub(MODAL_CONTENT_INSET.saturating_mul(2)),
            TerminalStyle::modal(),
        );
        let confirm = self.i18n.text("button-confirm");
        let cancel = self.i18n.text("button-cancel");
        let combined = button_width(&confirm)
            .saturating_add(1)
            .saturating_add(button_width(&cancel));
        let mut x = modal.x + modal.width.saturating_sub(combined) / 2;
        let y = modal.bottom().saturating_sub(MODAL_ACTION_BOTTOM_INSET);
        if let Some(rect) = draw_button(
            buffer,
            x,
            y,
            modal.right().saturating_sub(1),
            &confirm,
            TerminalStyle::destructive_button(),
        ) {
            self.targets.push(Target {
                area: rect,
                action: Action::Confirm,
            });
            x = rect.right().saturating_add(1);
        }
        if let Some(rect) = draw_button(
            buffer,
            x,
            y,
            modal.right().saturating_sub(1),
            &cancel,
            TerminalStyle::button(),
        ) {
            self.targets.push(Target {
                area: rect,
                action: Action::Cancel,
            });
        }
    }

    fn render_settings(&mut self, buffer: &mut Buffer, area: Rect) {
        let modal = centered(area, SETTINGS_WIDTH, SETTINGS_HEIGHT);
        draw_box(buffer, modal, TerminalStyle::modal());
        put_text(
            buffer,
            modal.x.saturating_add(MODAL_CONTENT_INSET),
            modal.y.saturating_add(SETTINGS_TITLE_ROW),
            &self.i18n.text("settings-title"),
            modal
                .width
                .saturating_sub(MODAL_CONTENT_INSET.saturating_mul(2)),
            TerminalStyle::modal(),
        );
        self.render_setting_row(
            buffer,
            modal,
            modal.y.saturating_add(SETTINGS_HEIGHT_ROW),
            "settings-height",
            self.config.default_height,
            Action::SettingsHeightLess,
            Action::SettingsHeightMore,
        );
        let done = self.i18n.text("button-done");
        let done_width = button_width(&done);
        if let Some(rect) = draw_button(
            buffer,
            modal.x + modal.width.saturating_sub(done_width) / 2,
            modal.bottom().saturating_sub(MODAL_ACTION_BOTTOM_INSET),
            modal.right().saturating_sub(1),
            &done,
            TerminalStyle::button(),
        ) {
            self.targets.push(Target {
                area: rect,
                action: Action::SettingsDone,
            });
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_setting_row(
        &mut self,
        buffer: &mut Buffer,
        modal: Rect,
        y: u16,
        id: &str,
        value: u16,
        less_action: Action,
        more_action: Action,
    ) {
        let mut args = FluentArgs::new();
        args.set("value", i64::from(value));
        put_text(
            buffer,
            modal.x.saturating_add(MODAL_CONTENT_INSET),
            y,
            &self.i18n.format(id, Some(&args)),
            modal.width.saturating_sub(SETTINGS_LABEL_RIGHT_INSET),
            TerminalStyle::modal(),
        );
        let less = self.i18n.text("button-decrease");
        let more = self.i18n.text("button-increase");
        let controls_x = modal.right().saturating_sub(SETTINGS_CONTROL_RIGHT_INSET);
        if let Some(rect) = draw_button(
            buffer,
            controls_x,
            y,
            modal.right().saturating_sub(1),
            &less,
            TerminalStyle::button(),
        ) {
            self.targets.push(Target {
                area: rect,
                action: less_action,
            });
        }
        if let Some(rect) = draw_button(
            buffer,
            controls_x.saturating_add(SETTINGS_CONTROL_SPACING),
            y,
            modal.right().saturating_sub(1),
            &more,
            TerminalStyle::button(),
        ) {
            self.targets.push(Target {
                area: rect,
                action: more_action,
            });
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat)
            || self.pending.is_some()
            || self.settings_open
        {
            return;
        }
        let Some(index) = self.focused.filter(|index| *index < self.editors.len()) else {
            return;
        };
        let control = key.modifiers.contains(KeyModifiers::CONTROL);
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        let text_width = self
            .layouts
            .get(index)
            .map(|layout| layout.text_width)
            .unwrap_or(
                self.viewport
                    .width
                    .saturating_sub(
                        SCROLLBAR_WIDTH + NOTE_BORDER_WIDTH + NOTE_PADDING.saturating_mul(2),
                    )
                    .max(1),
            );
        let rows = visual_rows(self.editors[index].text(), text_width);

        let changed = match key.code {
            KeyCode::Char(character) if control => match character.to_ascii_lowercase() {
                'a' => {
                    self.editors[index].select_all();
                    false
                }
                'c' => {
                    self.copy_selection(index, false);
                    false
                }
                'x' => self.copy_selection(index, true),
                'v' => self.paste(index),
                'z' if shift => self.editors[index].redo(),
                'z' => self.editors[index].undo(),
                'y' => self.editors[index].redo(),
                _ => false,
            },
            KeyCode::Char(character) if !alt => self.editors[index].insert(&character.to_string()),
            KeyCode::Enter => self.editors[index].insert("\n"),
            KeyCode::Tab => self.editors[index].insert(TAB_SPACES),
            KeyCode::Backspace => self.editors[index].backspace(),
            KeyCode::Delete => self.editors[index].delete(),
            KeyCode::Left => {
                self.editors[index].move_horizontal(-1, shift, control);
                false
            }
            KeyCode::Right => {
                self.editors[index].move_horizontal(1, shift, control);
                false
            }
            KeyCode::Up => {
                self.editors[index].move_vertical(&rows, -1, shift);
                false
            }
            KeyCode::Down => {
                self.editors[index].move_vertical(&rows, 1, shift);
                false
            }
            KeyCode::Home if control => {
                self.editors[index].move_document_edge(false, shift);
                false
            }
            KeyCode::End if control => {
                self.editors[index].move_document_edge(true, shift);
                false
            }
            KeyCode::Home => {
                self.editors[index].move_line_edge(&rows, false, shift);
                false
            }
            KeyCode::End => {
                self.editors[index].move_line_edge(&rows, true, shift);
                false
            }
            KeyCode::PageUp => {
                self.editors[index].move_page(
                    &rows,
                    usize::from(self.viewport.height.max(1)),
                    false,
                    shift,
                );
                false
            }
            KeyCode::PageDown => {
                self.editors[index].move_page(
                    &rows,
                    usize::from(self.viewport.height.max(1)),
                    true,
                    shift,
                );
                false
            }
            _ => false,
        };
        if changed {
            self.mark_dirty();
        }
        self.reveal_cursor(index);
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::ScrollUp => {
                self.scroll = self.scroll.saturating_sub(SCROLL_STEP);
                return;
            }
            MouseEventKind::ScrollDown => {
                self.scroll = self
                    .scroll
                    .saturating_add(SCROLL_STEP)
                    .min(self.max_scroll());
                return;
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                self.handle_drag(mouse.column, mouse.row);
                return;
            }
            MouseEventKind::Up(MouseButton::Left) => {
                self.drag = None;
                return;
            }
            MouseEventKind::Down(MouseButton::Left) => {}
            _ => return,
        }

        let position = Position::new(mouse.column, mouse.row);
        if let Some(action) = self
            .targets
            .iter()
            .rev()
            .find(|target| target.area.contains(position))
            .map(|target| target.action.clone())
        {
            self.activate(action);
            return;
        }
        if self.pending.is_some() || self.settings_open {
            return;
        }
        if self.viewport.contains(position)
            && mouse.column == self.viewport.right().saturating_sub(1)
        {
            self.drag = Some(Drag::Scrollbar);
            self.set_scroll_from_mouse(mouse.row);
            return;
        }
        if !self.viewport.contains(position) {
            return;
        }
        if let Some((index, layout)) = self
            .layouts
            .iter()
            .copied()
            .enumerate()
            .find(|(_, layout)| self.point_in_note_interior(*layout, mouse.column, mouse.row))
        {
            let cursor = self.hit_test_clamped(index, layout, mouse.column, mouse.row);
            self.focused = Some(index);
            self.editors[index].set_cursor(cursor, false);
            self.drag = Some(Drag::Text {
                note: index,
                anchor: cursor,
            });
        }
    }

    fn handle_drag(&mut self, column: u16, row: u16) {
        let Some(drag) = self.drag.as_ref() else {
            return;
        };
        match *drag {
            Drag::Text { note, anchor } => {
                let Some(layout) = self.layouts.get(note).copied() else {
                    return;
                };
                if row < self.viewport.y {
                    self.scroll = self.scroll.saturating_sub(1);
                } else if row >= self.viewport.bottom() {
                    self.scroll = self.scroll.saturating_add(1).min(self.max_scroll());
                }
                let cursor = self.hit_test_clamped(note, layout, column, row);
                self.editors[note].set_selection(anchor, cursor);
            }
            Drag::Scrollbar => self.set_scroll_from_mouse(row),
        }
    }

    fn activate(&mut self, action: Action) {
        if self.pending.is_some() {
            match action {
                Action::Confirm => self.confirm_pending(),
                Action::Cancel => self.pending = None,
                _ => {}
            }
            return;
        }
        if self.settings_open {
            match action {
                Action::SettingsHeightLess => {
                    self.config.default_height = self
                        .config
                        .default_height
                        .saturating_sub(SIZE_STEP)
                        .max(MIN_HEIGHT);
                }
                Action::SettingsHeightMore => {
                    self.config.default_height = self
                        .config
                        .default_height
                        .saturating_add(SIZE_STEP)
                        .min(MAX_HEIGHT);
                }
                Action::SettingsDone => {
                    self.settings_open = false;
                    if let Err(error) = self.config.save() {
                        let mut args = FluentArgs::new();
                        args.set("error", error.to_string());
                        self.transient_error =
                            Some(self.i18n.format("status-config-error", Some(&args)));
                    } else {
                        self.transient_error = None;
                    }
                }
                _ => {}
            }
            return;
        }

        match action {
            Action::Add => {
                self.editors.push(Editor::new(String::new()));
                let index = self.editors.len() - 1;
                self.focused = Some(index);
                self.mark_dirty();
                self.scroll = u32::MAX;
            }
            Action::Settings => self.settings_open = true,
            Action::Restore => self.restore_deleted(),
            Action::DeleteAll => self.pending = Some(Pending::DeleteAll),
            Action::Quit => {
                if self.dirty_since.is_some() {
                    self.save();
                }
                if self.dirty_since.is_none() {
                    self.quit = true;
                }
            }
            Action::Clear(index) => self.pending = Some(Pending::Clear(index)),
            Action::Delete(index) => self.pending = Some(Pending::Delete(index)),
            _ => {}
        }
    }

    fn confirm_pending(&mut self) {
        let Some(pending) = self.pending.take() else {
            return;
        };
        match pending {
            Pending::Clear(index) if index < self.editors.len() => {
                let text = self.editors[index].text().to_owned();
                if self.editors[index].clear() {
                    self.push_deleted(Deleted::Clear { index, text });
                    self.focused = Some(index);
                    self.mark_dirty();
                }
            }
            Pending::Delete(index) if index < self.editors.len() => {
                let text = self.editors.remove(index).text().to_owned();
                self.push_deleted(Deleted::Note { index, text });
                self.focused = if self.editors.is_empty() {
                    None
                } else {
                    Some(index.min(self.editors.len() - 1))
                };
                self.mark_dirty();
            }
            Pending::DeleteAll if !self.editors.is_empty() => {
                let notes = self
                    .editors
                    .drain(..)
                    .map(|editor| editor.text().to_owned())
                    .collect();
                self.push_deleted(Deleted::All { notes });
                self.focused = None;
                self.scroll = 0;
                self.mark_dirty();
            }
            _ => {}
        }
    }

    fn restore_deleted(&mut self) {
        let Some(deleted) = self.deleted.pop() else {
            return;
        };
        match deleted {
            Deleted::Clear { index, text } => {
                if index < self.editors.len() {
                    self.editors[index] = Editor::new(text);
                    self.focused = Some(index);
                } else {
                    self.editors.push(Editor::new(text));
                    self.focused = Some(self.editors.len() - 1);
                }
            }
            Deleted::Note { index, text } => {
                let index = index.min(self.editors.len());
                self.editors.insert(index, Editor::new(text));
                self.focused = Some(index);
            }
            Deleted::All { notes } => {
                self.editors = notes.into_iter().map(Editor::new).collect();
                self.focused = (!self.editors.is_empty()).then_some(0);
            }
        }
        self.mark_dirty();
    }

    fn push_deleted(&mut self, deleted: Deleted) {
        self.deleted.push(deleted);
        if self.deleted.len() > DELETION_UNDO_LIMIT {
            self.deleted.remove(0);
        }
    }

    fn copy_selection(&mut self, index: usize, cut: bool) -> bool {
        let Some(selection) = self.editors[index].selected_text().map(str::to_owned) else {
            return false;
        };
        match self.clipboard.write_text(selection) {
            Ok(()) if cut => self.editors[index].delete(),
            Ok(()) => false,
            Err(error) => {
                self.clipboard_error(error.to_string());
                false
            }
        }
    }

    fn paste(&mut self, index: usize) -> bool {
        match self.clipboard.read_text() {
            Ok(text) => self.editors[index].insert(&normalize_paste(&text)),
            Err(error) => {
                self.clipboard_error(error.to_string());
                false
            }
        }
    }

    fn insert_text(&mut self, text: &str) {
        if self.pending.is_some() || self.settings_open {
            return;
        }
        let Some(index) = self.focused.filter(|index| *index < self.editors.len()) else {
            return;
        };
        if self.editors[index].insert(text) {
            self.mark_dirty();
            self.reveal_cursor(index);
        }
    }

    fn clipboard_error(&mut self, error: String) {
        let mut args = FluentArgs::new();
        args.set("error", error);
        self.transient_error = Some(self.i18n.format("status-clipboard-error", Some(&args)));
    }

    fn mark_dirty(&mut self) {
        self.dirty_since = Some(Instant::now());
        self.save_error = None;
        self.transient_error = None;
    }

    fn save(&mut self) {
        let document = self.document();
        match storage::save(&self.path, &document) {
            Ok(()) => {
                self.dirty_since = None;
                self.save_error = None;
            }
            Err(error) => {
                self.save_error = Some(error.to_string());
                self.dirty_since = Some(Instant::now());
            }
        }
    }

    fn document(&self) -> Document {
        Document {
            notes: self
                .editors
                .iter()
                .map(|editor| editor.text().to_owned())
                .collect(),
        }
    }

    fn reveal_cursor(&mut self, index: usize) {
        let Some(layout) = self.layouts.get(index).copied() else {
            return;
        };
        let rows = visual_rows(self.editors[index].text(), layout.text_width);
        let (row, _) = cursor_row_column(
            self.editors[index].text(),
            self.editors[index].cursor(),
            &rows,
        );
        let cursor_y = layout.text_top().saturating_add(row as u32);
        if cursor_y < self.scroll {
            self.scroll = cursor_y;
        } else {
            let bottom = self.scroll.saturating_add(u32::from(self.viewport.height));
            if cursor_y >= bottom {
                self.scroll = cursor_y
                    .saturating_add(1)
                    .saturating_sub(u32::from(self.viewport.height));
            }
        }
        self.clamp_scroll();
    }

    fn hit_test_clamped(&self, index: usize, layout: NoteLayout, column: u16, row: u16) -> usize {
        let absolute_y = self
            .scroll
            .saturating_add(u32::from(row.saturating_sub(self.viewport.y)));
        if absolute_y < layout.text_top() {
            return 0;
        }
        let rows = visual_rows(self.editors[index].text(), layout.text_width);
        let rendered_bottom = layout
            .text_top()
            .saturating_add(rows.len().min(u32::MAX as usize) as u32);
        if absolute_y >= rendered_bottom {
            return self.editors[index].text().len();
        }
        let visual_row = absolute_y.saturating_sub(layout.text_top()) as usize;
        let visual_column = column
            .saturating_sub(layout.text_x())
            .min(layout.text_width);
        self.editors[index].hit_test(&rows, visual_row, visual_column)
    }

    fn point_in_note_interior(&self, layout: NoteLayout, column: u16, row: u16) -> bool {
        let left = layout.x.saturating_add(NOTE_BORDER_THICKNESS);
        let right = layout
            .x
            .saturating_add(layout.width)
            .saturating_sub(NOTE_BORDER_THICKNESS);
        if column < left || column >= right {
            return false;
        }
        let absolute_y = self
            .scroll
            .saturating_add(u32::from(row.saturating_sub(self.viewport.y)));
        absolute_y >= layout.top.saturating_add(u32::from(NOTE_BORDER_THICKNESS))
            && absolute_y < layout.bottom_border()
    }

    fn layout_visible(&self, layout: NoteLayout) -> bool {
        let bottom = layout.top.saturating_add(layout.height());
        bottom > self.scroll
            && layout.top < self.scroll.saturating_add(u32::from(self.viewport.height))
    }

    fn screen_y(&self, absolute: u32) -> Option<u16> {
        let relative = i64::from(absolute) - i64::from(self.scroll);
        if relative < 0 || relative >= i64::from(self.viewport.height) {
            return None;
        }
        Some(self.viewport.y + relative as u16)
    }

    fn set_scroll_from_mouse(&mut self, row: u16) {
        if self.viewport.height == 0 {
            return;
        }
        let relative = u32::from(
            row.saturating_sub(self.viewport.y)
                .min(self.viewport.height.saturating_sub(1)),
        );
        let denominator = u32::from(self.viewport.height.saturating_sub(1)).max(1);
        self.scroll = self.max_scroll().saturating_mul(relative) / denominator;
    }

    fn max_scroll(&self) -> u32 {
        self.total_height
            .saturating_sub(u32::from(self.viewport.height))
    }

    fn clamp_scroll(&mut self) {
        self.scroll = self.scroll.min(self.max_scroll());
    }
}

fn normalize_paste(value: &str) -> String {
    value.replace("\r\n", "\n").replace('\r', "\n")
}

fn render_text_row(
    buffer: &mut Buffer,
    x: u16,
    y: u16,
    width: u16,
    text: &str,
    row: &VisualRow,
    selection: Option<&std::ops::Range<usize>>,
) {
    let mut column = 0_u16;
    for (offset, grapheme) in text[row.start..row.end].grapheme_indices(true) {
        let grapheme_start = row.start + offset;
        let grapheme_end = grapheme_start + grapheme.len();
        let grapheme_width = display_width(grapheme);
        if column.saturating_add(grapheme_width) > width {
            break;
        }
        let selected =
            selection.is_some_and(|range| range.start < grapheme_end && range.end > grapheme_start);
        put_text(
            buffer,
            x.saturating_add(column),
            y,
            display_grapheme(grapheme),
            grapheme_width,
            if selected {
                TerminalStyle::selection()
            } else {
                TerminalStyle::text()
            },
        );
        column = column.saturating_add(grapheme_width);
    }
}

fn draw_horizontal_border(
    buffer: &mut Buffer,
    x: u16,
    y: u16,
    width: u16,
    top: bool,
    style: Style,
) {
    if width < 2 {
        return;
    }
    let (left, right) = if top {
        (BORDER_TOP_LEFT, BORDER_TOP_RIGHT)
    } else {
        (BORDER_BOTTOM_LEFT, BORDER_BOTTOM_RIGHT)
    };
    put_cell(buffer, x, y, left, style);
    for column in 1..width.saturating_sub(1) {
        put_cell(buffer, x + column, y, BORDER_HORIZONTAL, style);
    }
    put_cell(buffer, x + width - 1, y, right, style);
}

fn draw_box(buffer: &mut Buffer, area: Rect, style: Style) {
    for y in area.y..area.bottom() {
        for x in area.x..area.right() {
            put_cell(buffer, x, y, " ", style);
        }
    }
    draw_horizontal_border(buffer, area.x, area.y, area.width, true, style);
    draw_horizontal_border(
        buffer,
        area.x,
        area.bottom().saturating_sub(1),
        area.width,
        false,
        style,
    );
    for y in area.y.saturating_add(1)..area.bottom().saturating_sub(1) {
        put_cell(buffer, area.x, y, BORDER_VERTICAL, style);
        put_cell(
            buffer,
            area.right().saturating_sub(1),
            y,
            BORDER_VERTICAL,
            style,
        );
    }
}

fn centered(area: Rect, preferred_width: u16, preferred_height: u16) -> Rect {
    let width = preferred_width
        .min(area.width.saturating_sub(MODAL_MARGIN))
        .max(MIN_RENDERED_NOTE_WIDTH);
    let height = preferred_height
        .min(area.height.saturating_sub(MODAL_MARGIN))
        .max(MIN_RENDERED_NOTE_WIDTH);
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    )
}

fn draw_button(
    buffer: &mut Buffer,
    x: u16,
    y: u16,
    right: u16,
    label: &str,
    style: Style,
) -> Option<Rect> {
    let width = button_width(label);
    if width == 0 || x.saturating_add(width) > right {
        return None;
    }
    let area = Rect::new(x, y, width, 1);
    buffer.set_style(area, style);
    for column in x..x + width {
        put_cell(buffer, column, y, " ", style);
    }
    put_text(
        buffer,
        x + BUTTON_HORIZONTAL_PADDING,
        y,
        label,
        width.saturating_sub(BUTTON_HORIZONTAL_PADDING * 2),
        style,
    );
    Some(area)
}

fn button_width(label: &str) -> u16 {
    display_width(label).saturating_add(BUTTON_HORIZONTAL_PADDING * 2)
}

fn put_text(buffer: &mut Buffer, x: u16, y: u16, value: &str, width: u16, style: Style) {
    if width == 0 {
        return;
    }
    buffer.set_stringn(x, y, value, usize::from(width), style);
}

fn put_cell(buffer: &mut Buffer, x: u16, y: u16, value: &str, style: Style) {
    if let Some(cell) = buffer.cell_mut((x, y)) {
        cell.set_symbol(value).set_style(style);
    }
}

fn file_label(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_owned()
}
