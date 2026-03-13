use crate::utils::colorspace::Rgb;
use crate::utils::palette::LabeledColors;
use anyhow::{Ok, Result};
use ratatui::{
    Frame,
    crossterm::event::{self, Event, KeyCode, KeyModifiers},
    layout::{Constraint, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
};

// holds the palette data needed to render the preview tui
struct PreviewApp<'a> {
    palette: &'a [Rgb],
    name: &'a str,
    labels: &'a LabeledColors,
}

impl<'a> PreviewApp<'a> {
    fn new(palette: &'a [Rgb], name: &'a str, labels: &'a LabeledColors) -> Self {
        Self {
            palette,
            name,
            labels,
        }
    }

    /// iterate the palette and draw each one to screen. ratatui comes
    /// with these layouts, which ship areas to bind components to.
    /// https://ratatui.rs/concepts/layout/
    fn draw(&self, frame: &mut Frame) {
        let [title_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(frame.area());

        // title section which displays the filepath to the wallpaper
        let title = Paragraph::new(
            // line is not a literal line, like ----- which yes that is what i believed at first
            // it renders something (in this case the wallpaper path) in a single text line
            Line::from(self.name).centered(),
        )
        .block(Block::default());
        frame.render_widget(title, title_area);

        // main area which displays the swatches for each color in the palette
        // split into rows of 8 with square-ish cells
        self.draw_swatches(frame, main_area);

        // footer with information on how to leave the view (vim could never)
        let footer = Paragraph::new(Line::from("q/Esc to quit").centered())
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(footer, footer_area);
    }

    fn draw_swatches(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let cols_per_row = 8usize;
        let rows: Vec<&[Rgb]> = self.palette.chunks(cols_per_row).collect();
        let num_rows = rows.len();

        // terminal chars are ~2:1 height:width, so row height = col_width / 2 for squares
        let col_width = area.width as usize / cols_per_row;
        let row_height = (col_width / 2).max(1) as u16;

        let row_constraints: Vec<Constraint> = vec![Constraint::Length(row_height); num_rows];
        let row_areas = Layout::vertical(&row_constraints).split(area);

        for (row_idx, row_colors) in rows.iter().enumerate() {
            let col_constraints: Vec<Constraint> =
                vec![Constraint::Ratio(1, cols_per_row as u32); row_colors.len()];
            let col_areas = Layout::horizontal(&col_constraints).split(row_areas[row_idx]);

            for (col_idx, c) in row_colors.iter().enumerate() {
                let bg = Color::Rgb(c.0, c.1, c.2);
                let hex = c.hex();
                let label = self.labels.label_for(c);

                // vertically center the hex label within the swatch
                let content_lines: u16 = if label.is_some() { 2 } else { 1 };
                let padding = (row_height.saturating_sub(content_lines)) / 2;
                let mut lines: Vec<Line> = vec![Line::from(""); padding as usize];
                lines.push(
                    Line::from(Span::styled(hex, Style::default().fg(Color::Black).bg(bg)))
                        .centered(),
                );
                if let Some(name) = label {
                    lines.push(
                        Line::from(Span::styled(
                            format!("[{}]", name),
                            Style::default().fg(Color::DarkGray).bg(bg),
                        ))
                        .centered(),
                    );
                }

                let swatch = Paragraph::new(lines)
                    .style(Style::default().bg(bg))
                    .block(Block::default());

                frame.render_widget(swatch, col_areas[col_idx]);
            }
        }
    }
}

/// display the color palette to the user of the wallpaper that they provide
pub fn preview_palette(palette: &[Rgb], name: &str, labels: &LabeledColors) -> Result<()> {
    let app = PreviewApp::new(palette, name, labels);

    // tuis are event-based unlike video games where you render frames as fast as possible,
    // a tui only rerenders (generally) on some event
    ratatui::run(|terminal| {
        loop {
            terminal.draw(|frame| app.draw(frame))?;

            if let Event::Key(key) = event::read()? {
                if !key.is_press() {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    _ => {}
                }
            }
        }
        Ok(())
    })?;
    Ok(())
}
