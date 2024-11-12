use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    cursor,
};
use std::io::{self, Write};

struct Editor {
    cursor_x: usize,
    cursor_y: usize,
    buffer: Vec<Vec<char>>,
}

impl Editor {
    fn new() -> Self {
        Self {
            cursor_x: 0,
            cursor_y: 0,
            buffer: vec![vec![]],
        }
    }

    fn run(&mut self) -> crossterm::Result<()> {
        enable_raw_mode()?; // Enable raw mode to capture all keystrokes.
        let mut stdout = io::stdout();
        
        // Enter the alternate screen buffer.
        execute!(stdout, EnterAlternateScreen)?;

        loop {
            self.render(&mut stdout)?;

            if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                match (code, modifiers) {
                    (KeyCode::Char('q'), KeyModifiers::CONTROL) => break, // Exit with Ctrl + Q
                    (KeyCode::Char(c), _) => self.insert_char(c),
                    (KeyCode::Backspace, _) => self.delete_char(),
                    (KeyCode::Enter, _) => self.newline(),
                    (KeyCode::Left, _) => if self.cursor_x > 0 { self.cursor_x -= 1 },
                    (KeyCode::Right, _) => if self.cursor_x < self.buffer[self.cursor_y].len() { self.cursor_x += 1 },
                    (KeyCode::Up, _) => if self.cursor_y > 0 { self.cursor_y -= 1; self.adjust_cursor_x() },
                    (KeyCode::Down, _) => if self.cursor_y < self.buffer.len() - 1 { self.cursor_y += 1; self.adjust_cursor_x() },
                    _ => {}
                }
            }
        }

        // Leave the alternate screen buffer and restore the original screen.
        execute!(stdout, LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    fn render(&self, stdout: &mut io::Stdout) -> crossterm::Result<()> {
        // Clear the screen by moving the cursor to the top-left and clearing the terminal.
        execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
        
        for (y, line) in self.buffer.iter().enumerate() {
            let line_str: String = line.iter().collect();
            execute!(stdout, cursor::MoveTo(0, y as u16))?;
            write!(stdout, "{}", line_str)?;
        }
        
        // Move the cursor to the current cursor position.
        execute!(stdout, cursor::MoveTo(self.cursor_x as u16, self.cursor_y as u16))?;
        stdout.flush()?;
        Ok(())
    }

    fn insert_char(&mut self, c: char) {
        if self.cursor_x == self.buffer[self.cursor_y].len() {
            self.buffer[self.cursor_y].push(c);
        } else {
            self.buffer[self.cursor_y].insert(self.cursor_x, c);
        }
        self.cursor_x += 1;
    }

    fn delete_char(&mut self) {
        if self.cursor_x > 0 {
            // Remove the character to the left of the cursor if not at the line start.
            self.buffer[self.cursor_y].remove(self.cursor_x - 1);
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            // Move to the previous line if at the start of the current line.
            let mut removed_line = self.buffer.remove(self.cursor_y);
            
            // Move cursor to the end of the previous line.
            self.cursor_y -= 1;
            self.cursor_x = self.buffer[self.cursor_y].len();
            
            // Append the removed line to the end of the previous line.
            self.buffer[self.cursor_y].append(&mut removed_line);
        }
    }

    fn newline(&mut self) {
        let new_line = {
            // Isolate the mutable borrow in its own scope.
            self.buffer[self.cursor_y].split_off(self.cursor_x)
        };
        
        self.cursor_y += 1;
        self.cursor_x = 0;
        
        // Insert the new line without conflicting borrows.
        self.buffer.insert(self.cursor_y, new_line);
    }
    

    fn adjust_cursor_x(&mut self) {
        if self.cursor_x > self.buffer[self.cursor_y].len() {
            self.cursor_x = self.buffer[self.cursor_y].len();
        }
    }
}

fn main() -> crossterm::Result<()> {
    let mut editor = Editor::new();
    editor.run()
}
