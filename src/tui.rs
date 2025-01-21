use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Text,
    widgets::{Block, Borders},
    DefaultTerminal, Frame,
};

use crate::{inst::Inst, Greg, REGS};

pub fn run_tui(greg: Greg) -> anyhow::Result<()> {
    let terminal = ratatui::init();
    let app_result = State::new(greg).run(terminal);
    // TODO: just make sure that greg returns a value when done
    defer::defer! { ratatui::restore() };
    app_result
}

struct State<'a> {
    editing: bool,
    curr_reg: usize,
    curr_buf: String,
    prev_regs: [u32; 32],
    greg: Greg<'a>,
}

impl<'a> State<'a> {
    fn new(greg: Greg<'a>) -> Self {
        Self {
            editing: false,
            curr_reg: 0,
            curr_buf: String::new(),
            prev_regs: Default::default(),
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

    fn draw_inst(&self, ip: usize, inst: Inst, rect: Rect, frame: &mut Frame, style: Style) {
        let layout = Layout::horizontal([Constraint::Length(12), Constraint::Fill(1)]).split(rect);
        frame.render_widget(
            Text::styled(format!("0x{:08x}", ip), style.fg(Color::DarkGray)),
            layout[0],
        );
        frame.render_widget(
            Text::styled(format!("{}", inst.decompile()), style),
            layout[1],
        );
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

        let block = title_block("Preview".into());
        let preview_inner = block.inner(layout[1]);
        frame.render_widget(block, layout[1]);

        let block = title_block("STDOUT".into());
        let stdout = block.inner(layout[2]);
        frame.render_widget(block, layout[2]);

        let registers = Layout::vertical([Constraint::Length(1); 32]).split(reg_inner);
        for (i, r) in registers.iter().enumerate() {
            let curr = self.curr_reg == i;
            let row = Layout::horizontal([Constraint::Fill(1); 2]).split(*r);
            let style = Style::default().fg(Color::Yellow);

            frame.render_widget(Text::styled(REGS[i], style), row[0]);

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

        let regs = Layout::vertical(vec![Constraint::Length(1); preview_inner.height as usize])
            .split(preview_inner);
        let style = Style::default()
            .fg(Color::Blue)
            .add_modifier(Modifier::ITALIC);

        frame.render_widget(
            Text::styled(format!("IP: 0x{:08x}", self.greg.pc), style),
            regs[0],
        );

        let n = (preview_inner.height / 2 - 1) as isize;

        let mut idx = 1;

        for i in -n..=-1 {
            if let Some((ip, inst)) = self.greg.inst_off(i) {
                let ip = ip + self.greg.text_start;
                self.draw_inst(ip, inst, regs[idx], frame, style);
            }
            idx += 1;
        }

        {
            let style = style.bg(Color::Blue).fg(Color::Black);
            let ip = self.greg.pc + self.greg.text_start;
            self.draw_inst(ip, self.greg.curr_inst(), regs[idx], frame, style);
            idx += 1;
        }

        for i in 1..=n {
            if let Some((ip, inst)) = self.greg.inst_off(i) {
                let ip = ip + self.greg.text_start;
                self.draw_inst(ip, inst, regs[idx], frame, style);
            }
            idx += 1;
        }

        let s = self.greg.stdout.as_ref().unwrap();
        let lines = s
            .lines()
            .rev()
            .take(stdout.height as usize)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>();
        let s = lines.join("\n");
        let nlines = lines.len();
        let s = Text::styled(
            format!("{}{}", "\n".repeat(stdout.height as usize - nlines), s),
            Style::new().gray(),
        );
        frame.render_widget(s, stdout);
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
