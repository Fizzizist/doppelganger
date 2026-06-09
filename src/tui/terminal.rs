use std::io;

use ratatui_crossterm::CrosstermBackend;

pub struct TuiGuard {
    terminal: ratatui::Terminal<CrosstermBackend<io::Stdout>>,
}

impl TuiGuard {
    pub fn init() -> crate::error::Result<Self> {
        crossterm::execute!(io::stdout(), crossterm::terminal::EnterAlternateScreen)
            .map_err(|e| crate::error::Error::Tui(e.to_string()))?;
        crossterm::terminal::enable_raw_mode()
            .map_err(|e| crate::error::Error::Tui(e.to_string()))?;
        let backend = CrosstermBackend::new(io::stdout());
        let terminal =
            ratatui::Terminal::new(backend).map_err(|e| crate::error::Error::Tui(e.to_string()))?;
        Ok(Self { terminal })
    }

    pub fn terminal(&mut self) -> &mut ratatui::Terminal<CrosstermBackend<io::Stdout>> {
        &mut self.terminal
    }

    pub fn suspend(&mut self) -> crate::error::Result<()> {
        self.terminal
            .flush()
            .map_err(|e| crate::error::Error::Tui(e.to_string()))?;
        crossterm::terminal::disable_raw_mode()
            .map_err(|e| crate::error::Error::Tui(e.to_string()))?;
        crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen)
            .map_err(|e| crate::error::Error::Tui(e.to_string()))?;
        Ok(())
    }

    pub fn resume(&mut self) -> crate::error::Result<()> {
        crossterm::execute!(io::stdout(), crossterm::terminal::EnterAlternateScreen)
            .map_err(|e| crate::error::Error::Tui(e.to_string()))?;
        crossterm::terminal::enable_raw_mode()
            .map_err(|e| crate::error::Error::Tui(e.to_string()))?;
        let size =
            crossterm::terminal::size().map_err(|e| crate::error::Error::Tui(e.to_string()))?;
        let rect = ratatui::layout::Rect::new(0, 0, size.0, size.1);
        self.terminal.resize(rect)?;
        Ok(())
    }
}

impl Drop for TuiGuard {
    fn drop(&mut self) {
        let _ = self.terminal.flush();
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen);
    }
}
