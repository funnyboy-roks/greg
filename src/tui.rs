use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders},
    DefaultTerminal, Frame,
};

use crate::{
    decomp::{Addr, Decomp, DecompKind},
    reg::Reg,
    Greg, REGS,
};

pub fn run_tui(greg: Greg) -> anyhow::Result<()> {
    let terminal = ratatui::init();
    let app_result = State::new(greg).run(terminal);
    // TODO: just make sure that greg returns a value when done
    defer::defer! { ratatui::restore() };
    app_result
}

struct State {
    editing: bool,
    curr_reg: usize,
    curr_buf: String,
    prev_regs: [u32; 32],
    greg: Greg,
    decomp: Vec<Decomp>,
}

impl State {
    fn new(greg: Greg) -> Self {
        Self {
            editing: false,
            curr_reg: 0,
            curr_buf: String::new(),
            prev_regs: Default::default(),
            decomp: greg.decompile(),
            greg,
        }
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> anyhow::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            match event::read()? {
                Event::Key(key) => {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }

                    match key.code {
                        KeyCode::Char('j') if !self.editing => {
                            self.curr_reg = self.curr_reg.saturating_add(1);
                        }
                        KeyCode::Char('k') if !self.editing => {
                            self.curr_reg = self.curr_reg.saturating_sub(1);
                        }
                        KeyCode::Enter if self.editing => {
                            self.editing = false;
                            self.greg.reg[self.curr_reg] =
                                u32::from_str_radix(&self.curr_buf, 16).unwrap();
                            self.curr_buf.clear();
                        }
                        KeyCode::Enter if !self.editing => {
                            self.editing = true;
                            let curr = self.greg.reg[self.curr_reg];
                            if curr != 0 {
                                self.curr_buf = format!("{:x}", curr);
                            }
                        }
                        KeyCode::Esc if self.editing => {
                            self.editing = false;
                            self.curr_buf.clear();
                        }
                        KeyCode::Backspace if self.editing => {
                            self.curr_buf.pop();
                        }
                        KeyCode::Char('n') if !self.editing => {
                            self.prev_regs.copy_from_slice(&self.greg.reg);
                            self.greg.step();
                        }
                        KeyCode::Char(c @ '0'..='9' | c @ 'a'..='f' | c @ 'A'..='F')
                            if self.editing =>
                        {
                            if self.curr_buf.len() == 0 && c == '0' || self.curr_buf.len() >= 8 {
                                continue;
                            }
                            self.curr_buf.push(c.to_ascii_lowercase());
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
            let curr = self.curr_reg == i;
            let row = Layout::horizontal([Constraint::Fill(1); 2]).split(*r);
            let style = Style::default().fg(Color::Yellow);

            frame.render_widget(Reg::from(i as u32).into_span(), row[0]);

            let style = style.fg(Color::Gray).add_modifier(Modifier::ITALIC);
            if curr {
                let style = style.fg(Color::Black);
                if self.editing {
                    frame.render_widget(
                        Text::styled(format!("0x{:0>8}", &self.curr_buf), style.bg(Color::Green)),
                        row[1],
                    );
                } else {
                    frame.render_widget(
                        Text::styled(
                            format!("0x{:08x}", self.greg.reg[i]),
                            style.bg(Color::Magenta),
                        ),
                        row[1],
                    );
                }
            } else {
                let style = if self.greg.reg[i] != self.prev_regs[i] {
                    style.fg(Color::Black).bg(Color::Yellow)
                } else if self.greg.reg[i] == 0 {
                    style.fg(Color::DarkGray)
                } else {
                    style
                };
                frame.render_widget(
                    Text::styled(format!("0x{:08x}", self.greg.reg[i]), style),
                    row[1],
                );
            }
        }
    }

    fn draw_lines(&self, frame: &mut Frame, rect: Rect) {
        let regs = Layout::vertical(vec![Constraint::Length(1); rect.height as usize]).split(rect);
        let style = Style::default()
            .fg(Color::Blue)
            .add_modifier(Modifier::ITALIC);

        let n = rect.height as isize / 2 - 1;

        let curr = self
            .decomp
            .iter()
            .enumerate()
            .filter(|(_, d)| match d.kind {
                DecompKind::Label(_) => false,
                _ => d.addr >= self.greg.ip, // >= in-case ip is not actually a statement for some reason
            })
            .next();

        let mut idx;

        if let Some((curr, active)) = curr {
            let active_label = active.active_label();

            idx = n.checked_sub_unsigned(curr - 1).unwrap().max(0) as usize;
            let curr = curr as isize; // TODO: I dislike this `as isize`
            for i in (curr - n..=curr - 1).filter(|n| *n > 0) {
                let i = i as usize;
                self.draw_inst(&self.decomp[i], regs[idx], frame, style, active_label);
                idx += 1;
            }

            {
                let style = style.bg(Color::Indexed(237));
                self.draw_inst(active, regs[idx], frame, style, active_label);
                idx += 1;
            }

            let curr = curr as usize;
            for i in curr + 1..std::cmp::min(curr + n as usize, self.decomp.len()) {
                self.draw_inst(&self.decomp[i], regs[idx], frame, style, active_label);
                idx += 1;
            }
        } else {
            todo!()
        }
    }

    fn draw_stdout(&self, frame: &mut Frame, rect: Rect) {
        let s = self.greg.stdout.as_ref().unwrap();
        let lines = s
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
                label.into(),
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
                label.into(),
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
            vec![
                INDENT.into(),
                o.inst_name().into(),
                " ".into(),
                label.into(),
            ]
        }
    };
    Line::from(values)
}
