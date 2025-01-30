use std::{
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    thread,
    time::{Duration, Instant},
};

use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Borders},
    DefaultTerminal, Frame,
};

use crate::{
    decomp::{Addr, Decomp, DecompKind},
    reg::Reg,
    Greg, InstructionResult,
};

static DURATION: AtomicU64 = AtomicU64::new(1000);
static PLAY: AtomicBool = AtomicBool::new(false);
static STEP: AtomicBool = AtomicBool::new(false);

fn tock() {
    loop {
        let start = Instant::now();

        STEP.store(PLAY.load(Ordering::Relaxed), Ordering::Relaxed);

        let wait_time = Duration::from_millis(DURATION.load(Ordering::Relaxed));
        if let Some(remaining) = wait_time.checked_sub(start.elapsed()) {
            thread::sleep(remaining);
        }
    }
}

pub fn run_tui(greg: Greg) -> anyhow::Result<()> {
    let terminal = ratatui::init();
    std::thread::spawn(tock);
    let app_result = State::new(greg).run(terminal);
    ratatui::restore();
    app_result
}

#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum DisplayMode {
    Hex = 16,
    Dec = 10,
}

impl DisplayMode {
    fn radix(self) -> u32 {
        self as u32
    }
}

fn get_showable_char(c: u32) -> Option<char> {
    if !(0..=255).contains(&c) {
        return None;
    }

    let c = c as u8 as char;

    match c {
        '\n' | ' ' | '\t' => Some(c),
        c if c.is_ascii_graphic() => Some(c),
        _ => None,
    }
}

struct State {
    editing: bool,
    curr_reg: usize,
    curr_buf: u32,
    prev_regs: [u32; 32],
    greg: Greg,
    decomp: Vec<Decomp>,
    halt: bool,
    display_mode: DisplayMode,
}

impl State {
    fn new(greg: Greg) -> Self {
        Self {
            editing: false,
            curr_reg: 0,
            curr_buf: 0,
            prev_regs: Default::default(),
            decomp: greg.decompile(),
            greg,
            halt: false,
            display_mode: DisplayMode::Hex,
        }
    }

    fn step(&mut self) {
        if !self.halt {
            self.prev_regs.copy_from_slice(&self.greg.reg);
            match self.greg.step() {
                InstructionResult::None => {}
                InstructionResult::Done => {
                    self.halt = true;
                    PLAY.store(false, Ordering::Relaxed);
                }
                InstructionResult::Exit(_) => {
                    self.halt = true;
                    PLAY.store(false, Ordering::Relaxed);
                }
            }
        }
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> anyhow::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            if STEP.swap(false, Ordering::Relaxed) {
                self.step();
            }
            if !event::poll(Duration::from_millis(10))? {
                continue;
            }
            match event::read()? {
                Event::Key(key) => {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }

                    match key.code {
                        KeyCode::Char('d') if !self.editing => self.display_mode = DisplayMode::Dec,
                        KeyCode::Char('x') if !self.editing => self.display_mode = DisplayMode::Hex,
                        KeyCode::Char('j') | KeyCode::Down if !self.editing => {
                            self.curr_reg = self.curr_reg.saturating_add(1);
                        }
                        KeyCode::Char('k') | KeyCode::Up if !self.editing => {
                            self.curr_reg = self.curr_reg.saturating_sub(1);
                        }
                        KeyCode::Char('+') if !self.editing => {
                            DURATION.fetch_add(100, Ordering::Relaxed);
                        }
                        KeyCode::Char('-') if !self.editing => {
                            let prev = DURATION.load(Ordering::Relaxed);
                            let _ = DURATION.compare_exchange(
                                prev,
                                prev.saturating_sub(100),
                                Ordering::Relaxed,
                                Ordering::Relaxed,
                            );
                        }
                        KeyCode::Char(' ') if !self.editing => {
                            PLAY.fetch_not(Ordering::Relaxed);
                        }
                        KeyCode::Enter if self.editing => {
                            self.editing = false;
                            self.greg.reg[self.curr_reg] = self.curr_buf;
                            self.curr_buf = 0;
                        }
                        KeyCode::Enter if !self.editing => {
                            self.editing = true;
                            self.curr_buf = self.greg.reg[self.curr_reg];
                        }
                        KeyCode::Esc if self.editing => {
                            self.editing = false;
                            self.curr_buf = 0;
                        }
                        KeyCode::Backspace if self.editing => {
                            self.curr_buf /= self.display_mode.radix();
                        }
                        KeyCode::Char('n') if !self.editing => {
                            self.step();
                        }
                        KeyCode::Char(c)
                            if self.editing && c.is_digit(self.display_mode.radix()) =>
                        {
                            if let Some(curr_buf) =
                                self.curr_buf.checked_mul(self.display_mode.radix())
                            {
                                let ls = if c.is_ascii_digit() {
                                    (c as u8 - b'0') as u32
                                } else {
                                    let c = c.to_ascii_lowercase();
                                    (c as u8 - b'a') as u32 + 10
                                };

                                self.curr_buf = curr_buf + ls;
                            }
                        }
                        KeyCode::Char('q') if !self.editing => {
                            return Ok(());
                        }
                        _ => {}
                    }
                }
                e => {
                    dbg!(e);
                }
            }
        }
    }

    fn draw_inst(
        &self,
        decomp: &Decomp,
        rect: Rect,
        frame: &mut Frame,
        style: Style,
        active_label: Option<&str>,
    ) {
        let layout = Layout::horizontal([Constraint::Length(12), Constraint::Fill(1)]).split(rect);
        frame.render_widget(
            Text::styled(format!("0x{:08x}", decomp.addr), style.fg(Color::DarkGray)),
            layout[0],
        );
        frame.render_widget(render_decomp(decomp, active_label).style(style), layout[1]);
    }

    fn draw_registers(&self, frame: &mut Frame, rect: Rect) {
        let registers = Layout::vertical([Constraint::Length(1); 32]).split(rect);
        for (i, r) in registers.iter().enumerate() {
            let row = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Length(12),
                Constraint::Length(5),
            ])
            .split(*r);

            frame.render_widget(Reg::from(i as u32).into_span(), row[0]);

            let style = Style::default()
                .fg(Color::Black)
                .add_modifier(Modifier::ITALIC);
            let (style, n) = if self.curr_reg == i {
                if self.editing {
                    (style.bg(Color::Green), self.curr_buf)
                } else {
                    (style.bg(Color::Magenta), self.greg.reg[i])
                }
            } else {
                let style = if self.greg.reg[i] != self.prev_regs[i] {
                    style.bg(Color::Yellow)
                } else if self.greg.reg[i] == 0 {
                    style.fg(Color::DarkGray)
                } else {
                    style.fg(Color::Gray)
                };
                (style, self.greg.reg[i])
            };
            frame.render_widget(
                Text::styled(
                    match self.display_mode {
                        DisplayMode::Hex => format!(" 0x{:08x} ", n),
                        DisplayMode::Dec => format!(" {:10} ", n),
                    },
                    style,
                ),
                row[1],
            );
            if let Some(c) = get_showable_char(n) {
                frame.render_widget(
                    Text::styled(format!(" {:?}", c), Style::default().fg(Color::Green)),
                    row[2],
                );
            }
        }
    }

    fn draw_lines(&self, frame: &mut Frame, rect: Rect) {
        let regs = Layout::vertical(vec![Constraint::Length(1); rect.height as usize]).split(rect);
        let style = Style::default()
            .fg(Color::Blue)
            .add_modifier(Modifier::ITALIC);

        let before = rect.height as usize / 4;

        let curr = self.decomp.iter().enumerate().find(|(_, d)| match d.kind {
            DecompKind::Label(_) => false,
            _ => d.addr >= self.greg.ip, // >= in-case ip is not actually a statement for some reason
        });

        if let Some((curr, active)) = curr {
            let active_label = active.active_label();

            let start = curr.saturating_sub(before);

            for i in 0..rect.height as usize {
                if i + start >= self.decomp.len() {
                    break;
                }
                let style = if i + start == curr {
                    style.bg(Color::Indexed(237))
                } else {
                    style
                };
                self.draw_inst(&self.decomp[i + start], regs[i], frame, style, active_label);
            }
        } else {
            todo!()
        }
    }

    fn draw_stdout(&self, frame: &mut Frame, rect: Rect) {
        // TODO: Fix this
        let lines = self
            .greg
            .stdout
            .as_ref()
            .unwrap()
            .lines()
            .rev()
            .take(rect.height as usize)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>();
        let s = lines.join("\n");
        let nlines = lines.len();
        let s = Text::styled(
            format!("{}{}", "\n".repeat(rect.height as usize - nlines), s),
            Style::new().gray(),
        );
        frame.render_widget(s, rect);
    }

    fn draw(&self, frame: &mut Frame) {
        let layout = Layout::horizontal([
            Constraint::Ratio(1, 6),
            Constraint::Fill(1),
            Constraint::Ratio(1, 4),
        ])
        .spacing(2)
        .split(frame.area());

        let block = title_block("Registers".into());
        let reg_inner = block.inner(layout[0]);
        frame.render_widget(block, layout[0]);
        self.draw_registers(frame, reg_inner);

        let block = title_block("Preview".into());
        let preview_inner = block.inner(layout[1]);
        frame.render_widget(block, layout[1]);
        self.draw_lines(frame, preview_inner);

        let block = title_block("STDOUT".into());
        let stdout = block.inner(layout[2]);
        frame.render_widget(block, layout[2]);
        self.draw_stdout(frame, stdout);
    }
}

fn title_block(title: String) -> Block<'static> {
    Block::new()
        .borders(Borders::ALL)
        .title_alignment(Alignment::Center)
        .border_style(Style::new().dark_gray())
        .title_style(Style::new().reset())
        .title(title)
}

const INDENT: &str = "    ";

fn render_decomp<'a>(decomp: &'a Decomp, active_label: Option<&'a str>) -> Line<'a> {
    let values = match &decomp.kind {
        DecompKind::Syscall => vec![INDENT.into(), "syscall".fg(Color::Magenta)],
        DecompKind::Nop => vec![INDENT.into(), "nop".fg(Color::DarkGray)],
        DecompKind::Label(l) => {
            vec![
                if Some(l.as_str()) == active_label {
                    l.to_string().fg(Color::Yellow).bg(Color::Indexed(237))
                } else {
                    l.to_string().fg(Color::Yellow)
                },
                ":".fg(Color::Gray),
            ]
        }
        DecompKind::ArithLog { f, d, s, t } => {
            vec![
                INDENT.into(),
                f.inst_name().into(),
                " ".into(),
                d.into(),
                ", ".into(),
                s.into(),
                ", ".into(),
                t.into(),
            ]
        }
        DecompKind::DivMult { f, s, t } => {
            vec![
                INDENT.into(),
                f.inst_name().into(),
                " ".into(),
                s.into(),
                ", ".into(),
                t.into(),
            ]
        }
        DecompKind::Shift { f, d, t, a } => {
            vec![
                INDENT.into(),
                f.inst_name().into(),
                " ".into(),
                d.into(),
                ", ".into(),
                t.into(),
                ", ".into(),
                a.to_string().into(),
            ]
        }
        DecompKind::ShiftV { f, d, t, s } => {
            vec![
                INDENT.into(),
                f.inst_name().into(),
                " ".into(),
                d.into(),
                ", ".into(),
                t.into(),
                ", ".into(),
                s.into(),
            ]
        }
        DecompKind::JumpR { f, s } => {
            vec![INDENT.into(), f.inst_name().into(), " ".into(), s.into()]
        }
        DecompKind::MoveFrom { f, d } => {
            vec![INDENT.into(), f.inst_name().into(), " ".into(), d.into()]
        }
        DecompKind::MoveTo { f, s } => {
            vec![INDENT.into(), f.inst_name().into(), " ".into(), s.into()]
        }
        DecompKind::ArithLogI { o, t, s, i } => {
            vec![
                INDENT.into(),
                o.inst_name().into(),
                " ".into(),
                t.into(),
                ", ".into(),
                s.into(),
                ", ".into(),
                i.to_string().into(),
            ]
        }
        DecompKind::LoadI { o, t, imm } => {
            vec![
                INDENT.into(),
                o.inst_name().into(),
                " ".into(),
                t.into(),
                ", ".into(),
                imm.to_string().into(),
            ]
        }
        DecompKind::Branch { o, s, t, pos } => {
            let label = match pos {
                Addr::Label(l) => l.to_string().fg(Color::Yellow),
                Addr::Relative(n) => n.to_string().into(),
            };
            vec![
                INDENT.into(),
                o.inst_name().into(),
                " ".into(),
                s.into(),
                ", ".into(),
                t.into(),
                ", ".into(),
                label,
            ]
        }
        DecompKind::BranchZ { o, s, pos } => {
            let label = match pos {
                Addr::Label(l) => l.to_string().fg(Color::Yellow),
                Addr::Relative(n) => n.to_string().into(),
            };
            vec![
                INDENT.into(),
                o.inst_name().into(),
                " ".into(),
                s.into(),
                ", ".into(),
                label,
            ]
        }
        DecompKind::LoadStore { o, s, t, i } => {
            vec![
                INDENT.into(),
                o.inst_name().fg(Color::Red),
                " ".into(),
                t.into(),
                ", ".into(),
                i.to_string().into(),
                "(".into(),
                s.into(),
                ")".into(),
            ]
        }
        DecompKind::Jump { o, pos } => {
            let label = match pos {
                Addr::Label(l) => l.to_string().fg(Color::Yellow),
                Addr::Relative(n) => n.to_string().into(),
            };
            vec![INDENT.into(), o.inst_name().into(), " ".into(), label]
        }
    };
    Line::from(values)
}
