//! Shared presentation session exposed to QML as a QObject.

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;

        include!("cpp/app_identity.h");
        fn configure_app_identity();
        fn apply_window_icons();
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, pdf_url)]
        #[qproperty(i32, page_count)]
        #[qproperty(i32, current_page)]
        #[qproperty(i32, next_page)]
        #[qproperty(i32, audience_page)]
        #[qproperty(bool, blank)]
        #[qproperty(bool, frozen)]
        #[qproperty(i32, mode)]
        #[qproperty(f64, pointer_x)]
        #[qproperty(f64, pointer_y)]
        #[qproperty(bool, pointer_visible)]
        #[qproperty(f64, pointer_size)]
        #[qproperty(QString, current_strokes_json)]
        #[qproperty(QString, audience_strokes_json)]
        #[qproperty(QString, laser_trails_json)]
        #[qproperty(QString, page_label)]
        #[qproperty(QString, timer_text)]
        #[qproperty(bool, timer_paused)]
        #[qproperty(bool, goto_active)]
        #[qproperty(QString, goto_buffer)]
        #[qproperty(i32, presenter_screen)]
        #[qproperty(i32, audience_screen)]
        #[qproperty(bool, windowed)]
        #[qproperty(bool, help_visible)]
        #[qproperty(bool, history_visible)]
        #[qproperty(QString, history_targets_json)]
        #[qproperty(bool, current_on_stack)]
        #[qproperty(bool, next_on_stack)]
        #[qproperty(bool, recent_visible)]
        #[qproperty(QString, recent_files_json)]
        #[qproperty(QString, app_version)]
        #[qproperty(QString, git_id)]
        #[namespace = "pdfpc"]
        type Session = super::SessionRust;

        #[qinvokable]
        #[cxx_name = "loadPdf"]
        fn load_pdf(self: Pin<&mut Self>, path: &QString);

        #[qinvokable]
        #[cxx_name = "showRecent"]
        fn show_recent(self: Pin<&mut Self>);

        #[qinvokable]
        #[cxx_name = "hideRecent"]
        fn hide_recent(self: Pin<&mut Self>);

        #[qinvokable]
        #[cxx_name = "applyPageCount"]
        fn apply_page_count(self: Pin<&mut Self>, count: i32);

        #[qinvokable]
        #[cxx_name = "nextPage"]
        fn next_page_action(self: Pin<&mut Self>);

        #[qinvokable]
        #[cxx_name = "prevPage"]
        fn prev_page_action(self: Pin<&mut Self>);

        #[qinvokable]
        #[cxx_name = "gotoFirst"]
        fn goto_first(self: Pin<&mut Self>);

        #[qinvokable]
        #[cxx_name = "gotoLast"]
        fn goto_last(self: Pin<&mut Self>);

        #[qinvokable]
        #[cxx_name = "toggleBlank"]
        fn toggle_blank(self: Pin<&mut Self>);

        #[qinvokable]
        #[cxx_name = "toggleFreeze"]
        fn toggle_freeze(self: Pin<&mut Self>);

        #[qinvokable]
        #[cxx_name = "jumpToFrozen"]
        fn jump_to_frozen(self: Pin<&mut Self>);

        #[qinvokable]
        #[cxx_name = "exitState"]
        fn exit_state(self: Pin<&mut Self>);

        #[qinvokable]
        #[cxx_name = "switchMode"]
        fn switch_mode(self: Pin<&mut Self>, mode: i32);

        #[qinvokable]
        #[cxx_name = "clearDrawings"]
        fn clear_drawings(self: Pin<&mut Self>);

        #[qinvokable]
        #[cxx_name = "togglePause"]
        fn toggle_pause(self: Pin<&mut Self>);

        #[qinvokable]
        #[cxx_name = "startTimer"]
        fn start_timer(self: Pin<&mut Self>);

        #[qinvokable]
        #[cxx_name = "resetTimer"]
        fn reset_timer(self: Pin<&mut Self>);

        #[qinvokable]
        fn tick(self: Pin<&mut Self>);

        #[qinvokable]
        #[cxx_name = "swapScreens"]
        fn swap_screens(self: Pin<&mut Self>);

        #[qinvokable]
        #[cxx_name = "handleKey"]
        fn handle_key(self: Pin<&mut Self>, key: i32, modifiers: i32, text: &QString);

        #[qinvokable]
        #[cxx_name = "pointerMove"]
        fn pointer_move(self: Pin<&mut Self>, x: f64, y: f64);

        #[qinvokable]
        #[cxx_name = "pointerHide"]
        fn pointer_hide(self: Pin<&mut Self>);

        #[qinvokable]
        #[cxx_name = "laserTrailBegin"]
        fn laser_trail_begin(self: Pin<&mut Self>, x: f64, y: f64, straight: bool);

        #[qinvokable]
        #[cxx_name = "laserTrailAdd"]
        fn laser_trail_add(self: Pin<&mut Self>, x: f64, y: f64, straight: bool);

        #[qinvokable]
        #[cxx_name = "laserTrailEnd"]
        fn laser_trail_end(self: Pin<&mut Self>);

        #[qinvokable]
        #[cxx_name = "strokeBegin"]
        fn stroke_begin(self: Pin<&mut Self>, x: f64, y: f64);

        #[qinvokable]
        #[cxx_name = "strokeAdd"]
        fn stroke_add(self: Pin<&mut Self>, x: f64, y: f64);

        #[qinvokable]
        #[cxx_name = "strokeEnd"]
        fn stroke_end(self: Pin<&mut Self>);

        #[qinvokable]
        #[cxx_name = "mouseClick"]
        fn mouse_click(self: Pin<&mut Self>, button: i32);

        /// Follow an internal PDF link to `page` (0-based), pushing history.
        #[qinvokable]
        #[cxx_name = "followLink"]
        fn follow_link(self: Pin<&mut Self>, page: i32);

        #[qinvokable]
        #[cxx_name = "historyBack"]
        fn history_back(self: Pin<&mut Self>);

        #[qinvokable]
        #[cxx_name = "historyForward"]
        fn history_forward(self: Pin<&mut Self>);

        #[qinvokable]
        #[cxx_name = "toggleHistory"]
        fn toggle_history(self: Pin<&mut Self>);

        #[qinvokable]
        #[cxx_name = "hideHistory"]
        fn hide_history(self: Pin<&mut Self>);

        #[qinvokable]
        #[cxx_name = "configureWindowed"]
        fn configure_windowed(self: Pin<&mut Self>, windowed: bool);

        #[qinvokable]
        #[cxx_name = "applyWindowIcons"]
        fn refresh_window_icons(self: Pin<&mut Self>);
    }
}

use crate::laser_trail::{self, MIN_POINT_GAP};
use crate::nav_history::{format_elapsed, NavHistory, HISTORY_PICKER_MAX};
use crate::recent::{self, RecentEntry};
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QString, QUrl};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Mode: 0 = normal, 1 = laser, 2 = pen.
pub const MODE_NORMAL: i32 = 0;
pub const MODE_LASER: i32 = 1;
pub const MODE_PEN: i32 = 2;

const KEY_ESCAPE: i32 = 0x0100_0000;
const KEY_BACKSPACE: i32 = 0x0100_0003;
const KEY_RETURN: i32 = 0x0100_0004;
const KEY_ENTER: i32 = 0x0100_0005;
const KEY_HOME: i32 = 0x0100_0010;
const KEY_END: i32 = 0x0100_0011;
const KEY_LEFT: i32 = 0x0100_0012;
const KEY_UP: i32 = 0x0100_0013;
const KEY_RIGHT: i32 = 0x0100_0014;
const KEY_DOWN: i32 = 0x0100_0015;
const KEY_PAGE_UP: i32 = 0x0100_0016;
const KEY_PAGE_DOWN: i32 = 0x0100_0017;
const KEY_F3: i32 = 0x0100_0032;
const KEY_SPACE: i32 = 0x20;
const KEY_0: i32 = 0x30;
const KEY_9: i32 = 0x39;
const KEY_A: i32 = 0x41;
const KEY_Z: i32 = 0x5a;

/// Classic pdfpc: Shift+← / Shift+→ skips this many slides.
const PAGE_SKIP: i32 = 10;

/// Laser pointer radius as a fraction of the slide height.
/// Default is ~1/3 of the earlier 0.022 size so the tip matches a fine laser.
const POINTER_SIZE_DEFAULT: f64 = 0.022 / 3.0;
const POINTER_SIZE_MIN: f64 = 0.003;
const POINTER_SIZE_MAX: f64 = 0.08;
const POINTER_SIZE_STEP: f64 = 1.25;

/// Laser afterglow trail lifetime (left-button drag in laser mode).
const LASER_TRAIL_TTL: Duration = Duration::from_secs(2);

const SHIFT_MODIFIER: i32 = 0x0200_0000;
const CTRL_MODIFIER: i32 = 0x0400_0000;

/// Letter that stores the current slide on the jump stack without moving.
const STACK_STORE_KEY: char = 'm';

/// How a page leave interacts with jump history.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum LeaveKind {
    /// Go-to (`g`) or a PDF link: truncate the forward tail and push the page you leave.
    Stack,
    /// J-menu pick: truncate only the forward tail. Does not push a new page.
    Menu,
    /// Arrows, space, Shift±10, Home/End, and the other step keys. No stack change.
    Step,
    /// Ctrl+←/→ history walk — the stacks move in history_back/forward.
    History,
}

/// What `go_to_page` should do to the stack for a [`LeaveKind`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum StackAction {
    /// Sequential paging and history walks (the walk already updated the deques).
    None,
    /// Go-to / PDF link.
    RecordAndTruncate,
    /// J-menu: drop the forward tail only.
    TruncateOnly,
}

fn stack_action(kind: LeaveKind) -> StackAction {
    match kind {
        LeaveKind::Stack => StackAction::RecordAndTruncate,
        LeaveKind::Menu => StackAction::TruncateOnly,
        LeaveKind::Step | LeaveKind::History => StackAction::None,
    }
}

type Point = (f64, f64);
type Stroke = Vec<Point>;

/// One mouse-drag polyline (or straight segment). Fade starts only on release.
struct LaserStroke {
    points: Vec<Point>,
    start: Point,
    straight: bool,
    /// `None` while the button is held; set in `laser_trail_end`.
    fade_started: Option<Instant>,
}

pub struct SessionRust {
    pdf_url: QString,
    page_count: i32,
    current_page: i32,
    next_page: i32,
    audience_page: i32,
    blank: bool,
    frozen: bool,
    mode: i32,
    pointer_x: f64,
    pointer_y: f64,
    pointer_visible: bool,
    pointer_size: f64,
    current_strokes_json: QString,
    audience_strokes_json: QString,
    laser_trails_json: QString,
    page_label: QString,
    timer_text: QString,
    timer_paused: bool,
    goto_active: bool,
    goto_buffer: QString,
    presenter_screen: i32,
    audience_screen: i32,
    windowed: bool,
    help_visible: bool,
    history_visible: bool,
    history_targets_json: QString,
    current_on_stack: bool,
    next_on_stack: bool,
    recent_visible: bool,
    recent_files_json: QString,
    app_version: QString,
    git_id: QString,

    strokes: HashMap<i32, Vec<Stroke>>,
    drawing: bool,
    laser_trails: Vec<LaserStroke>,
    laser_drawing: bool,
    timer_started: bool,
    timer_paused_flag: bool,
    accumulated: Duration,
    last_tick: Option<Instant>,
    /// Absolute filesystem path of the open PDF (empty if none).
    pdf_path: String,
    /// Page to restore once PdfDocument reports pageCount.
    pending_page: Option<i32>,
    recent_entries: Vec<RecentEntry>,
    /// Jump history. Sequential steps do not push; go-to, links, and `m` do.
    nav_history: NavHistory,
    /// Last time the open J-menu refreshed its visited-ago labels.
    history_menu_refreshed: Option<Instant>,
}

impl Default for SessionRust {
    fn default() -> Self {
        Self {
            pdf_url: QString::default(),
            page_count: 0,
            current_page: 0,
            next_page: -1,
            audience_page: 0,
            blank: false,
            frozen: false,
            mode: MODE_NORMAL,
            pointer_x: -1.0,
            pointer_y: -1.0,
            pointer_visible: false,
            pointer_size: POINTER_SIZE_DEFAULT,
            current_strokes_json: QString::from("[]"),
            audience_strokes_json: QString::from("[]"),
            laser_trails_json: QString::from("[]"),
            page_label: QString::from("0 / 0"),
            timer_text: QString::from("00:00:00"),
            timer_paused: false,
            goto_active: false,
            goto_buffer: QString::default(),
            presenter_screen: 0,
            audience_screen: 1,
            windowed: false,
            help_visible: false,
            history_visible: false,
            history_targets_json: QString::from("[]"),
            current_on_stack: false,
            next_on_stack: false,
            recent_visible: false,
            recent_files_json: QString::from("[]"),
            app_version: QString::from(env!("PDFPC_VERSION")),
            git_id: QString::from(env!("PDFPC_GIT_ID")),
            strokes: HashMap::new(),
            drawing: false,
            laser_trails: Vec::new(),
            laser_drawing: false,
            timer_started: false,
            timer_paused_flag: false,
            accumulated: Duration::ZERO,
            last_tick: None,
            pdf_path: String::new(),
            pending_page: None,
            recent_entries: recent::load(),
            nav_history: NavHistory::default(),
            history_menu_refreshed: None,
        }
    }
}

impl qobject::Session {
    pub fn load_pdf(mut self: Pin<&mut Self>, path: &QString) {
        self.as_mut().persist_current_page();

        let path_str = path.to_string();
        let absolute = resolve_absolute_path(&path_str);
        let abs_str = absolute.to_string_lossy().into_owned();
        let url = QUrl::from_user_input(&QString::from(abs_str.as_str()), &QString::default())
            .to_qstring();

        let restore_page = recent::page_for(&self.rust().recent_entries, &abs_str);

        {
            let mut rust = self.as_mut().rust_mut();
            rust.pdf_path = abs_str.clone();
            rust.pending_page = Some(restore_page);
            rust.strokes.clear();
            rust.drawing = false;
            rust.laser_trails.clear();
            rust.laser_drawing = false;
            rust.timer_started = false;
            rust.timer_paused_flag = false;
            rust.accumulated = Duration::ZERO;
            rust.last_tick = None;
            rust.nav_history.clear();
            recent::touch(&mut rust.recent_entries, &abs_str, restore_page);
            recent::save(&rust.recent_entries);
        }

        // Reset UI state before setting pdf_url: changing the URL can load the
        // document re-entrantly (apply_page_count) before this function returns.
        self.as_mut().set_page_count(0);
        self.as_mut().set_current_page(restore_page);
        self.as_mut().set_blank(false);
        self.as_mut().set_frozen(false);
        self.as_mut().set_mode(MODE_NORMAL);
        self.as_mut().set_timer_paused(false);
        self.as_mut().set_timer_text(QString::from("00:00:00"));
        self.as_mut().set_help_visible(false);
        self.as_mut().set_history_visible(false);
        self.as_mut().set_history_targets_json(QString::from("[]"));
        self.as_mut().set_recent_visible(false);
        self.as_mut().set_goto_active(false);
        self.as_mut().set_goto_buffer(QString::default());
        self.as_mut().sync_recent_json();
        self.as_mut().pointer_hide();
        self.as_mut().sync_laser_trails();
        self.as_mut().refresh_derived();
        self.as_mut().set_pdf_url(url);
    }

    pub fn show_recent(mut self: Pin<&mut Self>) {
        self.as_mut().persist_current_page();
        {
            let mut rust = self.as_mut().rust_mut();
            rust.recent_entries = recent::load();
        }
        self.as_mut().sync_recent_json();
        self.as_mut().set_help_visible(false);
        self.as_mut().set_history_visible(false);
        self.set_recent_visible(true);
    }

    pub fn hide_recent(self: Pin<&mut Self>) {
        // ESC continues the current PDF; without one the dialog stays open.
        if !self.rust().pdf_path.is_empty() {
            self.set_recent_visible(false);
        }
    }

    pub fn apply_page_count(mut self: Pin<&mut Self>, count: i32) {
        let page_count = count.max(0);
        // Ignore transient 0 from a document that is still loading / empty so
        // we neither wipe a good count nor consume pending_page early.
        if page_count <= 0 {
            return;
        }
        self.as_mut().set_page_count(page_count);
        let pending = {
            let mut rust = self.as_mut().rust_mut();
            rust.pending_page.take()
        };
        if let Some(page) = pending {
            self.as_mut()
                .set_current_page(page.clamp(0, page_count - 1));
        } else {
            let current = *self.current_page();
            if current >= page_count {
                self.as_mut().set_current_page(page_count - 1);
            }
        }
        self.as_mut().persist_current_page();
        self.refresh_derived();
    }

    pub fn next_page_action(self: Pin<&mut Self>) {
        self.jump_pages(1);
    }

    pub fn prev_page_action(self: Pin<&mut Self>) {
        self.jump_pages(-1);
    }

    /// Move by `delta` slides (negative = back). Clamps to the document.
    /// Matches classic pdfpc Shift+←/→ (±10) and plain ←/→ (±1).
    /// Sequential moves do not push the jump stack.
    fn jump_pages(self: Pin<&mut Self>, delta: i32) {
        if delta == 0 {
            return;
        }
        if *self.goto_active() {
            if delta > 0 {
                self.commit_goto();
            } else {
                self.cancel_goto();
            }
            return;
        }
        let count = *self.page_count();
        let cur = *self.current_page();
        let Some(target) = page_after_jump(cur, count, delta) else {
            return;
        };
        self.go_to_page(target, LeaveKind::Step);
    }

    pub fn goto_first(self: Pin<&mut Self>) {
        self.go_to_page(0, LeaveKind::Step);
    }

    pub fn goto_last(self: Pin<&mut Self>) {
        let count = *self.page_count();
        if count > 0 {
            self.go_to_page(count - 1, LeaveKind::Step);
        }
    }

    pub fn toggle_blank(self: Pin<&mut Self>) {
        let next = !*self.blank();
        self.set_blank(next);
    }

    pub fn toggle_freeze(mut self: Pin<&mut Self>) {
        let currently = *self.frozen();
        let cur = *self.current_page();
        if currently {
            self.as_mut().set_frozen(false);
            self.as_mut().set_audience_page(cur);
            self.sync_stroke_props();
        } else {
            self.as_mut().set_audience_page(cur);
            self.as_mut().set_frozen(true);
            self.sync_stroke_props();
        }
    }

    /// While frozen, jump the presenter view back to the audience (frozen) page.
    pub fn jump_to_frozen(self: Pin<&mut Self>) {
        if !*self.frozen() {
            return;
        }
        let target = *self.audience_page();
        self.go_to_page(target, LeaveKind::Step);
    }

    pub fn exit_state(mut self: Pin<&mut Self>) {
        if *self.history_visible() {
            self.hide_history();
            return;
        }
        if *self.goto_active() {
            self.cancel_goto();
            return;
        }
        if *self.blank() {
            self.as_mut().set_blank(false);
        }
        if *self.frozen() {
            let cur = *self.current_page();
            self.as_mut().set_frozen(false);
            self.as_mut().set_audience_page(cur);
            self.as_mut().sync_stroke_props();
        }
        if self.rust().timer_paused_flag {
            {
                let mut rust = self.as_mut().rust_mut();
                rust.timer_paused_flag = false;
                rust.last_tick = Some(Instant::now());
            }
            self.as_mut().set_timer_paused(false);
        }
        if *self.mode() != MODE_NORMAL {
            self.set_mode_internal(MODE_NORMAL);
        }
    }

    pub fn switch_mode(self: Pin<&mut Self>, mode: i32) {
        self.set_mode_internal(mode);
    }

    pub fn clear_drawings(mut self: Pin<&mut Self>) {
        let page = *self.current_page();
        self.as_mut().rust_mut().strokes.remove(&page);
        self.sync_stroke_props();
    }

    pub fn toggle_pause(mut self: Pin<&mut Self>) {
        if !self.rust().timer_started {
            return;
        }
        if self.rust().timer_paused_flag {
            {
                let mut rust = self.as_mut().rust_mut();
                rust.timer_paused_flag = false;
                rust.last_tick = Some(Instant::now());
            }
            self.as_mut().set_timer_paused(false);
        } else {
            let extra = self.rust().last_tick.map(|last| last.elapsed());
            {
                let mut rust = self.as_mut().rust_mut();
                if let Some(extra) = extra {
                    rust.accumulated += extra;
                }
                rust.timer_paused_flag = true;
                rust.last_tick = None;
            }
            self.as_mut().set_timer_paused(true);
        }
        self.update_timer_text();
    }

    /// Start the presentation timer if it is not already running (e.g. after reset).
    pub fn start_timer(mut self: Pin<&mut Self>) {
        if self.rust().timer_started {
            return;
        }
        {
            let mut rust = self.as_mut().rust_mut();
            rust.timer_started = true;
            rust.timer_paused_flag = false;
            rust.last_tick = Some(Instant::now());
        }
        self.as_mut().set_timer_paused(false);
        self.update_timer_text();
    }

    pub fn reset_timer(mut self: Pin<&mut Self>) {
        {
            let mut rust = self.as_mut().rust_mut();
            rust.timer_started = false;
            rust.timer_paused_flag = false;
            rust.accumulated = Duration::ZERO;
            rust.last_tick = None;
        }
        self.as_mut().set_timer_paused(false);
        self.set_timer_text(QString::from("00:00:00"));
    }

    pub fn tick(mut self: Pin<&mut Self>) {
        if self.rust().timer_started && !self.rust().timer_paused_flag {
            self.as_mut().update_timer_text();
        }
        // Keep afterglow opacity in sync even when the presentation timer is idle.
        if !self.rust().laser_trails.is_empty() {
            self.as_mut().sync_laser_trails();
        }
        if *self.history_visible() {
            let now = Instant::now();
            let due = self
                .rust()
                .history_menu_refreshed
                .map(|t| now.saturating_duration_since(t) >= Duration::from_secs(1))
                .unwrap_or(true);
            if due {
                self.as_mut().rust_mut().history_menu_refreshed = Some(now);
                self.as_mut().sync_history_targets_json();
            }
        }
    }

    pub fn swap_screens(mut self: Pin<&mut Self>) {
        let p = *self.presenter_screen();
        let a = *self.audience_screen();
        self.as_mut().set_presenter_screen(a);
        self.set_audience_screen(p);
    }

    pub fn handle_key(mut self: Pin<&mut Self>, key: i32, modifiers: i32, text: &QString) {
        let shift = modifiers & SHIFT_MODIFIER != 0;
        let ctrl = modifiers & CTRL_MODIFIER != 0;
        let text_str = text.to_string();
        let ch = text_str.chars().next();

        // Recent-files dialog: Escape continues the open PDF.
        if *self.recent_visible() {
            match key {
                KEY_ESCAPE | KEY_F3 => self.hide_recent(),
                _ => {}
            }
            return;
        }

        // Jump-history picker: digit selects a target; j / Escape closes.
        if *self.history_visible() {
            self.handle_history_picker_key(key, ch);
            return;
        }

        // While the help is shown only h / Escape / F3 do anything.
        if *self.help_visible() {
            let letter = ch
                .filter(|c| c.is_ascii_alphabetic())
                .map(|c| c.to_ascii_lowercase())
                .or_else(|| key_letter(key));
            if key == KEY_ESCAPE || letter == Some('h') {
                self.set_help_visible(false);
            } else if key == KEY_F3 {
                self.as_mut().set_help_visible(false);
                self.show_recent();
            }
            return;
        }

        if *self.goto_active() {
            self.handle_goto_key(key, &text_str);
            return;
        }

        if ctrl && key_letter(key) == Some('t') {
            self.reset_timer();
            return;
        }

        if ctrl && key == KEY_LEFT {
            self.history_back();
            return;
        }
        if ctrl && key == KEY_RIGHT {
            self.history_forward();
            return;
        }
        if ctrl && key_letter(key) == Some('g') {
            self.jump_to_frozen();
            return;
        }

        if shift && key_letter(key) == Some('s') {
            self.swap_screens();
            return;
        }

        if key == KEY_F3 {
            self.show_recent();
            return;
        }

        let mode = *self.mode();
        // Slide keys are consumed in every mode. Normal, laser, and pen apply
        // the same delta; mouse clicks stay mode-specific (pen draws).
        if nav_key_delta(key, shift).is_some() {
            if let Some(delta) = nav_delta_for_mode(mode, key, shift) {
                self.jump_pages(delta);
            }
            return;
        }
        match key {
            KEY_HOME => self.goto_first(),
            KEY_END => self.goto_last(),
            KEY_ESCAPE => self.exit_state(),
            _ => {
                // Prefer event text; fall back to Qt key codes so shortcuts still
                // work when the platform delivers empty text (seen on some Wayland IMEs).
                let letter = ch
                    .filter(|c| c.is_ascii_alphabetic())
                    .map(|c| c.to_ascii_lowercase())
                    .or_else(|| key_letter(key));
                if let Some(letter) = letter {
                    match letter {
                        'b' => self.toggle_blank(),
                        'f' => self.toggle_freeze(),
                        'g' => self.begin_goto(),
                        'c' => self.clear_drawings(),
                        'p' => self.toggle_pause(),
                        's' => self.start_timer(),
                        'h' => self.set_help_visible(true),
                        'j' => self.toggle_history(),
                        letter if letter == STACK_STORE_KEY => self.store_current_on_stack(),
                        _ => {}
                    }
                } else if let Some(ch) = ch {
                    match ch {
                        '1' => self.set_mode_internal(MODE_NORMAL),
                        '2' => self.set_mode_internal(MODE_LASER),
                        '3' => self.set_mode_internal(MODE_PEN),
                        '+' => self.scale_pointer(POINTER_SIZE_STEP),
                        '-' => self.scale_pointer(1.0 / POINTER_SIZE_STEP),
                        _ => {}
                    }
                } else if (KEY_0..=KEY_9).contains(&key) {
                    match (key - KEY_0) as u8 {
                        1 => self.set_mode_internal(MODE_NORMAL),
                        2 => self.set_mode_internal(MODE_LASER),
                        3 => self.set_mode_internal(MODE_PEN),
                        _ => {}
                    }
                }
            }
        }
    }

    pub fn pointer_move(mut self: Pin<&mut Self>, x: f64, y: f64) {
        if *self.mode() != MODE_LASER {
            return;
        }
        self.as_mut().set_pointer_x(x.clamp(0.0, 1.0));
        self.as_mut().set_pointer_y(y.clamp(0.0, 1.0));
        self.set_pointer_visible(true);
    }

    pub fn pointer_hide(mut self: Pin<&mut Self>) {
        self.as_mut().set_pointer_visible(false);
        self.as_mut().set_pointer_x(-1.0);
        self.set_pointer_y(-1.0);
    }

    pub fn laser_trail_begin(mut self: Pin<&mut Self>, x: f64, y: f64, straight: bool) {
        if *self.mode() != MODE_LASER {
            return;
        }
        let pt = (x.clamp(0.0, 1.0), y.clamp(0.0, 1.0));
        {
            let mut rust = self.as_mut().rust_mut();
            rust.laser_trails.push(LaserStroke {
                points: vec![pt],
                start: pt,
                straight,
                fade_started: None,
            });
            rust.laser_drawing = true;
        }
        self.sync_laser_trails();
    }

    pub fn laser_trail_add(mut self: Pin<&mut Self>, x: f64, y: f64, straight: bool) {
        if !self.rust().laser_drawing || *self.mode() != MODE_LASER {
            return;
        }
        let pt = (x.clamp(0.0, 1.0), y.clamp(0.0, 1.0));
        let mut changed = false;
        if let Some(last) = self.as_mut().rust_mut().laser_trails.last_mut() {
            last.straight = straight;
            if straight {
                laser_trail::set_straight_segment(&mut last.points, last.start, pt);
                changed = true;
            } else if laser_trail::append_vertex(&mut last.points, pt, MIN_POINT_GAP) {
                changed = true;
            }
        }
        if changed {
            self.sync_laser_trails();
        }
    }

    pub fn laser_trail_end(mut self: Pin<&mut Self>) {
        {
            let mut rust = self.as_mut().rust_mut();
            rust.laser_drawing = false;
            if let Some(last) = rust.laser_trails.last_mut() {
                if last.fade_started.is_none() {
                    last.fade_started = Some(Instant::now());
                }
            }
        }
        self.sync_laser_trails();
    }

    pub fn stroke_begin(mut self: Pin<&mut Self>, x: f64, y: f64) {
        if *self.mode() != MODE_PEN {
            return;
        }
        let page = *self.current_page();
        let pt = (x.clamp(0.0, 1.0), y.clamp(0.0, 1.0));
        {
            let mut rust = self.as_mut().rust_mut();
            rust.strokes.entry(page).or_default().push(vec![pt]);
            rust.drawing = true;
        }
        self.sync_stroke_props();
    }

    pub fn stroke_add(mut self: Pin<&mut Self>, x: f64, y: f64) {
        if !self.rust().drawing || *self.mode() != MODE_PEN {
            return;
        }
        let page = *self.current_page();
        let pt = (x.clamp(0.0, 1.0), y.clamp(0.0, 1.0));
        if let Some(strokes) = self.as_mut().rust_mut().strokes.get_mut(&page) {
            if let Some(last) = strokes.last_mut() {
                last.push(pt);
            }
        }
        self.sync_stroke_props();
    }

    pub fn stroke_end(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().drawing = false;
        self.sync_stroke_props();
    }

    pub fn mouse_click(self: Pin<&mut Self>, button: i32) {
        let mode = *self.mode();
        if *self.help_visible()
            || *self.recent_visible()
            || *self.history_visible()
            || (mode != MODE_NORMAL && mode != MODE_LASER)
        {
            return;
        }
        match button {
            1 => self.next_page_action(),
            2 => self.prev_page_action(),
            _ => {}
        }
    }

    pub fn follow_link(self: Pin<&mut Self>, page: i32) {
        if *self.help_visible() || *self.recent_visible() || *self.history_visible() {
            return;
        }
        self.stack_jump(page);
    }

    pub fn history_back(mut self: Pin<&mut Self>) {
        if *self.help_visible()
            || *self.recent_visible()
            || *self.history_visible()
            || *self.goto_active()
        {
            return;
        }
        let cur = *self.current_page();
        // Always keep the page we leave, so forward still works after back.
        let target = self
            .as_mut()
            .rust_mut()
            .nav_history
            .go_back(cur, Instant::now());
        if let Some(target) = target {
            self.apply_history_page(target);
        }
    }

    pub fn history_forward(mut self: Pin<&mut Self>) {
        if *self.help_visible()
            || *self.recent_visible()
            || *self.history_visible()
            || *self.goto_active()
        {
            return;
        }
        let cur = *self.current_page();
        let target = self
            .as_mut()
            .rust_mut()
            .nav_history
            .go_forward(cur, Instant::now());
        if let Some(target) = target {
            self.apply_history_page(target);
        }
    }

    pub fn toggle_history(mut self: Pin<&mut Self>) {
        if *self.history_visible() {
            self.hide_history();
            return;
        }
        if *self.help_visible() || *self.recent_visible() || *self.goto_active() {
            return;
        }
        self.as_mut().sync_history_targets_json();
        self.as_mut().rust_mut().history_menu_refreshed = Some(Instant::now());
        self.set_history_visible(true);
    }

    pub fn hide_history(self: Pin<&mut Self>) {
        self.set_history_visible(false);
    }

    pub fn configure_windowed(self: Pin<&mut Self>, windowed: bool) {
        self.set_windowed(windowed);
    }

    pub fn refresh_window_icons(self: Pin<&mut Self>) {
        qobject::apply_window_icons();
    }

    fn scale_pointer(self: Pin<&mut Self>, factor: f64) {
        let size = (*self.pointer_size() * factor).clamp(POINTER_SIZE_MIN, POINTER_SIZE_MAX);
        self.set_pointer_size(size);
    }

    fn set_mode_internal(mut self: Pin<&mut Self>, mode: i32) {
        let mode = match mode {
            MODE_LASER => MODE_LASER,
            MODE_PEN => MODE_PEN,
            _ => MODE_NORMAL,
        };
        self.as_mut().set_mode(mode);
        {
            let mut rust = self.as_mut().rust_mut();
            rust.drawing = false;
            rust.laser_drawing = false;
            if mode != MODE_LASER {
                rust.laser_trails.clear();
            }
        }
        if mode != MODE_LASER {
            self.as_mut().pointer_hide();
            self.sync_laser_trails();
        }
    }

    fn maybe_start_timer_on_leave_first(self: Pin<&mut Self>, from_page: i32) {
        if from_page == 0 && !self.rust().timer_started {
            self.start_timer();
        }
    }

    fn begin_goto(mut self: Pin<&mut Self>) {
        self.as_mut().set_goto_active(true);
        self.set_goto_buffer(QString::default());
    }

    fn cancel_goto(mut self: Pin<&mut Self>) {
        self.as_mut().set_goto_active(false);
        self.set_goto_buffer(QString::default());
    }

    fn commit_goto(mut self: Pin<&mut Self>) {
        let buf = self.goto_buffer().to_string();
        self.as_mut().cancel_goto();
        if let Ok(one_based) = buf.parse::<i32>() {
            let count = *self.page_count();
            if one_based >= 1 && one_based <= count {
                self.stack_jump(one_based - 1);
            }
        }
    }

    /// Go-to or PDF link: record the page being left and drop the forward tail.
    fn stack_jump(self: Pin<&mut Self>, target: i32) {
        self.go_to_page(target, LeaveKind::Stack);
    }

    /// J-menu pick: drop the forward tail, do not push the page being left.
    fn menu_jump(self: Pin<&mut Self>, target: i32) {
        self.go_to_page(target, LeaveKind::Menu);
    }

    /// Apply a page from Back/Forward without pushing a new jump entry.
    fn apply_history_page(self: Pin<&mut Self>, target: i32) {
        self.go_to_page(target, LeaveKind::History);
    }

    /// Store the current slide. Does not change page and does not drop forward history.
    fn store_current_on_stack(mut self: Pin<&mut Self>) {
        let count = *self.page_count();
        let cur = *self.current_page();
        if count <= 0 || cur < 0 || cur >= count {
            return;
        }
        self.as_mut()
            .rust_mut()
            .nav_history
            .toggle_page(cur, Instant::now());
        self.sync_stack_marks();
    }

    /// Navigate to `target`. Only [`LeaveKind::Stack`] pushes the page you leave.
    fn go_to_page(mut self: Pin<&mut Self>, target: i32, kind: LeaveKind) {
        let count = *self.page_count();
        if count <= 0 || target < 0 || target >= count {
            return;
        }
        let cur = *self.current_page();
        if target == cur {
            return;
        }
        let now = Instant::now();
        self.as_mut().rust_mut().nav_history.note_visit(cur, now);
        match stack_action(kind) {
            StackAction::None => {}
            StackAction::RecordAndTruncate => {
                self.as_mut()
                    .rust_mut()
                    .nav_history
                    .begin_new_jump(cur, now, true);
            }
            StackAction::TruncateOnly => {
                self.as_mut()
                    .rust_mut()
                    .nav_history
                    .begin_new_jump(cur, now, false);
            }
        }
        self.as_mut().rust_mut().nav_history.note_visit(target, now);
        self.as_mut().maybe_start_timer_on_leave_first(cur);
        self.as_mut().clear_laser_trails();
        self.as_mut().set_current_page(target);
        self.refresh_derived();
    }

    fn handle_history_picker_key(mut self: Pin<&mut Self>, key: i32, ch: Option<char>) {
        let letter = ch
            .filter(|c| c.is_ascii_alphabetic())
            .map(|c| c.to_ascii_lowercase())
            .or_else(|| key_letter(key));
        if key == KEY_ESCAPE || letter == Some('j') {
            self.hide_history();
            return;
        }
        let digit = history_picker_digit(key, ch);
        if let Some(digit) = digit {
            if (1..=HISTORY_PICKER_MAX).contains(&digit) {
                let idx = digit - 1;
                let cur = *self.current_page();
                let targets = self
                    .rust()
                    .nav_history
                    .picker_targets(HISTORY_PICKER_MAX, cur);
                if let Some(target) = targets.get(idx) {
                    let page = target.page;
                    self.as_mut().hide_history();
                    self.menu_jump(page);
                }
            }
        }
    }

    fn sync_history_targets_json(self: Pin<&mut Self>) {
        let now = Instant::now();
        let cur = *self.current_page();
        let targets = self
            .rust()
            .nav_history
            .picker_targets(HISTORY_PICKER_MAX, cur);
        let arr: Vec<Value> = targets
            .iter()
            .enumerate()
            .map(|(i, target)| {
                let elapsed = now.saturating_duration_since(target.last_visited);
                json!({
                    "key": (i + 1).to_string(),
                    "page": target.page,
                    "label": format!("Slide {}", target.page + 1),
                    "ago": format_elapsed(elapsed),
                })
            })
            .collect();
        self.set_history_targets_json(QString::from(&Value::Array(arr).to_string()));
    }

    fn handle_goto_key(self: Pin<&mut Self>, key: i32, text: &str) {
        match key {
            KEY_ESCAPE => self.cancel_goto(),
            KEY_RETURN | KEY_ENTER => self.commit_goto(),
            KEY_BACKSPACE => {
                let mut buf = self.goto_buffer().to_string();
                buf.pop();
                self.set_goto_buffer(QString::from(&buf));
            }
            _ => {
                let mut buf = self.goto_buffer().to_string();
                let mut changed = false;
                for ch in text.chars() {
                    if ch.is_ascii_digit() {
                        buf.push(ch);
                        changed = true;
                    }
                }
                if !changed && text.is_empty() && (KEY_0..=KEY_9).contains(&key) {
                    buf.push(char::from_u32((key - KEY_0) as u32 + u32::from(b'0')).unwrap());
                    changed = true;
                }
                if changed {
                    self.set_goto_buffer(QString::from(&buf));
                }
            }
        }
    }

    fn refresh_derived(mut self: Pin<&mut Self>) {
        let count = *self.page_count();
        let cur = *self.current_page();
        let next = if count > 0 && cur + 1 < count {
            cur + 1
        } else {
            -1
        };
        self.as_mut().set_next_page(next);

        if !*self.frozen() {
            self.as_mut().set_audience_page(cur);
        }

        let label = if count > 0 {
            format!("{} / {}", cur + 1, count)
        } else {
            "0 / 0".to_string()
        };
        self.as_mut().set_page_label(QString::from(&label));
        self.as_mut().persist_current_page();
        self.as_mut().sync_stroke_props();
        self.sync_stack_marks();
    }

    fn sync_stack_marks(mut self: Pin<&mut Self>) {
        let cur = *self.current_page();
        let next = *self.next_page();
        let cur_on = self.rust().nav_history.contains_page(cur);
        let next_on = next >= 0 && self.rust().nav_history.contains_page(next);
        self.as_mut().set_current_on_stack(cur_on);
        self.set_next_on_stack(next_on);
    }

    fn persist_current_page(mut self: Pin<&mut Self>) {
        let path = self.rust().pdf_path.clone();
        if path.is_empty() {
            return;
        }
        let page = *self.current_page();
        {
            let mut rust = self.as_mut().rust_mut();
            recent::set_page(&mut rust.recent_entries, &path, page);
            recent::save(&rust.recent_entries);
        }
    }

    fn sync_recent_json(self: Pin<&mut Self>) {
        let json = recent::to_json(&self.rust().recent_entries);
        self.set_recent_files_json(QString::from(&json));
    }

    fn sync_stroke_props(mut self: Pin<&mut Self>) {
        let cur = *self.current_page();
        let aud = *self.audience_page();
        let current_json = strokes_to_json(self.rust().strokes.get(&cur));
        let audience_json = strokes_to_json(self.rust().strokes.get(&aud));
        self.as_mut()
            .set_current_strokes_json(QString::from(&current_json));
        self.set_audience_strokes_json(QString::from(&audience_json));
    }

    fn clear_laser_trails(mut self: Pin<&mut Self>) {
        {
            let mut rust = self.as_mut().rust_mut();
            rust.laser_trails.clear();
            rust.laser_drawing = false;
        }
        self.sync_laser_trails();
    }

    fn sync_laser_trails(mut self: Pin<&mut Self>) {
        let now = Instant::now();
        {
            let mut rust = self.as_mut().rust_mut();
            rust.laser_trails.retain(|stroke| {
                laser_trail::stroke_alpha(stroke.fade_started, now, LASER_TRAIL_TTL.as_secs_f64())
                    .is_some()
            });
        }
        let json = laser_trails_to_json(&self.rust().laser_trails, now);
        self.set_laser_trails_json(QString::from(&json));
    }

    fn update_timer_text(self: Pin<&mut Self>) {
        let mut total = self.rust().accumulated;
        if let Some(last) = self.rust().last_tick {
            if !self.rust().timer_paused_flag {
                total += last.elapsed();
            }
        }
        let secs = total.as_secs();
        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        let s = secs % 60;
        let text = format!("{h:02}:{m:02}:{s:02}");
        self.set_timer_text(QString::from(&text));
    }
}

fn strokes_to_json(strokes: Option<&Vec<Stroke>>) -> String {
    let mut arr: Vec<Value> = Vec::new();
    if let Some(strokes) = strokes {
        for stroke in strokes {
            let pts: Vec<Value> = stroke
                .iter()
                .map(|(x, y)| json!({ "x": x, "y": y }))
                .collect();
            arr.push(Value::Array(pts));
        }
    }
    Value::Array(arr).to_string()
}

fn laser_trails_to_json(trails: &[LaserStroke], now: Instant) -> String {
    let ttl = LASER_TRAIL_TTL.as_secs_f64();
    let mut arr: Vec<Value> = Vec::new();
    for stroke in trails {
        if stroke.points.len() < 2 {
            continue;
        }
        let Some(a) = laser_trail::stroke_alpha(stroke.fade_started, now, ttl) else {
            continue;
        };
        let pts: Vec<Value> = stroke
            .points
            .iter()
            .map(|(x, y)| json!({ "x": x, "y": y }))
            .collect();
        arr.push(json!({ "a": a, "pts": pts }));
    }
    Value::Array(arr).to_string()
}

fn key_letter(key: i32) -> Option<char> {
    if (KEY_A..=KEY_Z).contains(&key) {
        Some(char::from_u32((key - KEY_A) as u32 + u32::from(b'a')).unwrap())
    } else {
        None
    }
}

/// Resolve a filesystem path or `file:` URL to an absolute path.
fn resolve_absolute_path(path_str: &str) -> PathBuf {
    let candidate = if path_str.starts_with("file:") {
        file_url_to_path(path_str)
    } else {
        PathBuf::from(path_str)
    };
    std::fs::canonicalize(&candidate).unwrap_or_else(|_| {
        if candidate.is_absolute() {
            candidate
        } else {
            std::env::current_dir()
                .map(|dir| dir.join(&candidate))
                .unwrap_or(candidate)
        }
    })
}

fn file_url_to_path(url: &str) -> PathBuf {
    let rest = url.strip_prefix("file:").unwrap_or(url);
    let path_part = if let Some(stripped) = rest.strip_prefix("///") {
        format!("/{stripped}")
    } else if let Some(stripped) = rest.strip_prefix("//localhost") {
        stripped.to_string()
    } else if rest.starts_with("//") {
        // file://host/path — drop the host component.
        match rest[2..].find('/') {
            Some(idx) => rest[2 + idx..].to_string(),
            None => rest.to_string(),
        }
    } else {
        rest.to_string()
    };
    PathBuf::from(percent_decode(&path_part))
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let h = hex_nibble(bytes[i + 1]);
            let l = hex_nibble(bytes[i + 2]);
            if let (Some(h), Some(l)) = (h, l) {
                out.push((h << 4) | l);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn history_picker_digit(key: i32, ch: Option<char>) -> Option<usize> {
    if let Some(c) = ch {
        if c.is_ascii_digit() {
            return Some((c as u8 - b'0') as usize);
        }
    }
    if (KEY_0..=KEY_9).contains(&key) {
        return Some((key - KEY_0) as usize);
    }
    None
}

/// Keyboard next/previous is the same in normal, laser, and pen.
/// Mouse-click navigation is separate and stays off while the pen is drawing.
fn keyboard_slide_nav(mode: i32) -> bool {
    mode == MODE_NORMAL || mode == MODE_LASER || mode == MODE_PEN
}

/// Relative slide delta for a navigation key. `None` if `key` is not one.
/// Shift scales only ←/→ (`PAGE_SKIP`); the other shortcuts stay ±1.
fn nav_key_delta(key: i32, shift: bool) -> Option<i32> {
    match key {
        KEY_RIGHT => Some(if shift { PAGE_SKIP } else { 1 }),
        KEY_LEFT => Some(if shift { -PAGE_SKIP } else { -1 }),
        KEY_DOWN | KEY_SPACE | KEY_RETURN | KEY_ENTER | KEY_PAGE_DOWN => Some(1),
        KEY_UP | KEY_BACKSPACE | KEY_PAGE_UP => Some(-1),
        _ => None,
    }
}

/// Slide delta applied for `mode`, or `None` when that mode ignores the key.
fn nav_delta_for_mode(mode: i32, key: i32, shift: bool) -> Option<i32> {
    if !keyboard_slide_nav(mode) {
        return None;
    }
    nav_key_delta(key, shift)
}

/// Target page after a relative jump, or `None` if nothing would change.
fn page_after_jump(cur: i32, count: i32, delta: i32) -> Option<i32> {
    if count <= 0 || delta == 0 {
        return None;
    }
    let target = (i64::from(cur) + i64::from(delta)).clamp(0, i64::from(count - 1)) as i32;
    if target == cur {
        None
    } else {
        Some(target)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        nav_delta_for_mode, page_after_jump, stack_action, LeaveKind, StackAction, KEY_BACKSPACE,
        KEY_DOWN, KEY_ENTER, KEY_ESCAPE, KEY_F3, KEY_HOME, KEY_LEFT, KEY_PAGE_DOWN, KEY_PAGE_UP,
        KEY_RETURN, KEY_RIGHT, KEY_SPACE, KEY_UP, MODE_LASER, MODE_NORMAL, MODE_PEN, PAGE_SKIP,
        STACK_STORE_KEY,
    };

    #[test]
    fn shift_skip_jumps_ten() {
        assert_eq!(page_after_jump(0, 50, PAGE_SKIP), Some(10));
        assert_eq!(page_after_jump(15, 50, -PAGE_SKIP), Some(5));
    }

    #[test]
    fn shift_skip_clamps_at_ends() {
        assert_eq!(page_after_jump(45, 50, PAGE_SKIP), Some(49));
        assert_eq!(page_after_jump(3, 50, -PAGE_SKIP), Some(0));
        assert_eq!(page_after_jump(0, 50, -PAGE_SKIP), None);
        assert_eq!(page_after_jump(49, 50, PAGE_SKIP), None);
    }

    #[test]
    fn single_step_still_works() {
        assert_eq!(page_after_jump(7, 50, 1), Some(8));
        assert_eq!(page_after_jump(7, 50, -1), Some(6));
    }

    #[test]
    fn pen_mode_slide_keys_match_other_modes() {
        let keys = [
            (KEY_RIGHT, false),
            (KEY_LEFT, false),
            (KEY_RIGHT, true),
            (KEY_LEFT, true),
            (KEY_DOWN, false),
            (KEY_SPACE, false),
            (KEY_RETURN, false),
            (KEY_ENTER, false),
            (KEY_PAGE_DOWN, false),
            (KEY_UP, false),
            (KEY_BACKSPACE, false),
            (KEY_PAGE_UP, false),
            (KEY_SPACE, true),
            (KEY_PAGE_UP, true),
        ];
        for (key, shift) in keys {
            let normal = nav_delta_for_mode(MODE_NORMAL, key, shift);
            let laser = nav_delta_for_mode(MODE_LASER, key, shift);
            let pen = nav_delta_for_mode(MODE_PEN, key, shift);
            assert!(normal.is_some(), "key {key:#x} shift={shift}");
            assert_eq!(laser, normal);
            assert_eq!(pen, normal);
            let delta = normal.unwrap();
            assert_eq!(
                page_after_jump(12, 50, delta),
                Some((12 + delta).clamp(0, 49))
            );
        }
        // Home / Escape / F3 are not relative slide keys in any mode.
        assert_eq!(nav_delta_for_mode(MODE_PEN, KEY_HOME, false), None);
        assert_eq!(nav_delta_for_mode(MODE_NORMAL, KEY_ESCAPE, false), None);
        assert_eq!(nav_delta_for_mode(MODE_LASER, KEY_F3, false), None);
        // An unknown mode does not inherit pen/laser/normal navigation.
        assert_eq!(nav_delta_for_mode(99, KEY_RIGHT, false), None);
        assert_eq!(nav_delta_for_mode(99, KEY_SPACE, false), None);
    }

    #[test]
    fn only_goto_and_links_push_the_stack() {
        assert_eq!(
            stack_action(LeaveKind::Stack),
            StackAction::RecordAndTruncate
        );
        assert_eq!(stack_action(LeaveKind::Menu), StackAction::TruncateOnly);
        assert_eq!(stack_action(LeaveKind::Step), StackAction::None);
        assert_eq!(stack_action(LeaveKind::History), StackAction::None);
    }

    #[test]
    fn store_key_does_not_collide_with_shortcuts() {
        // Letters already bound in handle_key. Ctrl+G, Ctrl+T, and Shift+S are chords.
        const TAKEN: &[char] = &['b', 'c', 'f', 'g', 'h', 'j', 'p', 's'];
        assert!(
            !TAKEN.contains(&STACK_STORE_KEY),
            "store key {STACK_STORE_KEY} is already a shortcut"
        );
        assert_eq!(STACK_STORE_KEY, 'm');
    }
}
