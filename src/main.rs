use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{self, Color, Colors, Stylize},
    terminal::{self, ClearType},
};
use std::io::{stdout, Write};
use std::path::PathBuf;
use std::vec::Vec;

#[derive(Debug)]
enum Mode {
    Normal,
    Insert,
}

struct Editor {
    mode: Mode,
    content: Vec<String>,
    cursor_x: usize,
    cursor_y: usize,
    filename: Option<PathBuf>,
    terminal_size: (u16, u16),
}

impl Editor {
    fn new() -> Editor {
        Editor {
            mode: Mode::Normal,
            content: vec![String::new()],
            cursor_x: 0,
            cursor_y: 0,
            filename: None,
            terminal_size: terminal::size().unwrap_or((80, 24)),
        }
    }

    fn run(&mut self) -> crossterm::Result<()> {
        terminal::enable_raw_mode()?;
        execute!(stdout(), terminal::EnterAlternateScreen)?;

        loop {
            self.terminal_size = terminal::size()?;
            self.refresh_screen()?;

            if let Event::Key(event) = event::read()? {
                if let Err(_) = self.handle_keypress(event) {
                    break;
                }
            }
        }

        execute!(stdout(), terminal::LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    fn refresh_screen(&mut self) -> crossterm::Result<()> {
        let mut stdout = stdout();
        queue!(
            stdout,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )?;

        // Calculate maximum line number width
        let line_num_width = (self.content.len() + 1).to_string().len();
        
        // Display content with line numbers
        for (i, line) in self.content.iter().enumerate() {
            let line_num = i + 1;
            queue!(
                stdout,
                style::SetColors(Colors::new(Color::DarkGrey, Color::Black)),
                cursor::MoveTo(0, i as u16),
                style::Print(format!("{:>width$} │ ", line_num, width = line_num_width)),
                style::SetColors(Colors::new(Color::Reset, Color::Reset)),
                style::Print(line),
                style::Print("\r\n")
            )?;
        }

        // Status bar (bottom line)
        let status_bar_y = self.terminal_size.1 - 2;
        let file_name = self.filename
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("[No Name]");
        
        let status = format!(
            " {} - Line {}/{}, Col {} ", 
            file_name,
            self.cursor_y + 1,
            self.content.len(),
            self.cursor_x + 1
        );

        let mode_str = format!(" {:?} MODE ", self.mode);
        let padding = " ".repeat(
            self.terminal_size.0 as usize 
            - status.len() 
            - mode_str.len()
        );

        queue!(
            stdout,
            cursor::MoveTo(0, status_bar_y),
            style::SetColors(Colors::new(Color::Black, Color::White)),
            style::Print(&status),
            style::Print(padding),
            style::Print(&mode_str),
            style::SetColors(Colors::new(Color::Reset, Color::Reset)),
        )?;

        // Help line
        queue!(
            stdout,
            cursor::MoveTo(0, status_bar_y + 1),
            style::SetColors(Colors::new(Color::DarkGrey, Color::Reset)),
            style::Print(" CTRL-Q: Quit | i: Insert Mode | ESC: Normal Mode"),
            style::SetColors(Colors::new(Color::Reset, Color::Reset))
        )?;

        // Move cursor to current position (accounting for line number margin)
        queue!(
            stdout,
            cursor::MoveTo(
                (line_num_width + 3 + self.cursor_x) as u16,
                self.cursor_y as u16
            )
        )?;

        stdout.flush()?;
        Ok(())
    }

    fn handle_keypress(&mut self, event: KeyEvent) -> crossterm::Result<()> {
        match self.mode {
            Mode::Normal => self.handle_normal_mode(event),
            Mode::Insert => self.handle_insert_mode(event),
        }
    }

    fn handle_normal_mode(&mut self, event: KeyEvent) -> crossterm::Result<()> {
        match event.code {
            KeyCode::Char('i') => self.mode = Mode::Insert,
            KeyCode::Char('h') => self.move_cursor_left(),
            KeyCode::Char('j') => self.move_cursor_down(),
            KeyCode::Char('k') => self.move_cursor_up(),
            KeyCode::Char('l') => self.move_cursor_right(),
            KeyCode::Char('q') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Exit requested",
                )
                .into())
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_insert_mode(&mut self, event: KeyEvent) -> crossterm::Result<()> {
        match event.code {
            KeyCode::Esc => self.mode = Mode::Normal,
            KeyCode::Char(c) => self.insert_char(c),
            KeyCode::Enter => self.insert_newline(),
            KeyCode::Backspace => self.handle_backspace(),
            _ => {}
        }
        Ok(())
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        }
    }

    fn move_cursor_right(&mut self) {
        if self.cursor_x < self.content[self.cursor_y].len() {
            self.cursor_x += 1;
        }
    }

    fn move_cursor_up(&mut self) {
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = std::cmp::min(self.cursor_x, self.content[self.cursor_y].len());
        }
    }

    fn move_cursor_down(&mut self) {
        if self.cursor_y < self.content.len() - 1 {
            self.cursor_y += 1;
            self.cursor_x = std::cmp::min(self.cursor_x, self.content[self.cursor_y].len());
        }
    }

    fn insert_char(&mut self, c: char) {
        let x = self.cursor_x;
        self.content[self.cursor_y].insert(x, c);
        self.cursor_x += 1;
    }

    fn insert_newline(&mut self) {
        let y = self.cursor_y;
        let x = self.cursor_x;
        let current_line = self.content[y][x..].to_string();
        self.content[y].truncate(x);
        self.content.insert(y + 1, current_line);
        self.cursor_y += 1;
        self.cursor_x = 0;
    }

    fn handle_backspace(&mut self) {
        if self.cursor_x > 0 {
            let x = self.cursor_x;
            self.content[self.cursor_y].remove(x - 1);
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            let y = self.cursor_y;
            let current_line = self.content.remove(y);
            self.cursor_y -= 1;
            let previous_len = self.content[self.cursor_y].len();
            self.content[self.cursor_y].push_str(&current_line);
            self.cursor_x = previous_len;
        }
    }
}

fn main() -> crossterm::Result<()> {
    let mut editor = Editor::new();
    editor.run()
}