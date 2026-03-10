use crate::utils::rgb::Rgb;
use anyhow::{Ok, Result};
use ratatui::{
    Frame,
    crossterm::event::{self, Event, KeyCode, KeyModifiers},
    layout::{Constraint, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
};

/// Display the color palette to the user of the wallpaper that they
/// provide
pub fn preview_palette(palette: &[Rgb], name: &str) -> Result<()> {
    // refs used:
    // https://ratatui.rs/tutorials/counter-app/basic-app/
    // we can optionally pull this terminal value which is a crossterm type
    // by default
    // Then, use the draw method on the terminal type to call *our* draw method.
    // in the reference link above they use a struct + impl setup for this,
    // for now i just want this to work then I will refactor to struct + impl setup
    // TODO: refactor to struct / impl setup as seen in link above
    ratatui::run(|terminal| {
        loop {
            terminal.draw(|frame| draw(frame, palette, name))?;

            // one thing to note is that unlike video games, tuis are 'event' based
            // so while my traditional understanding of graphics from video games is just
            // render frames as fast as possible, a tui only rerenders (generally) on some event
            if let Event::Key(key) = event::read()? {
                if !key.is_press() {
                    continue;
                }
                match key.code {
                    // q => quit
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    // ctrl c => quit
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    _ => {}
                }
            }
        }
        Ok(())
    })?;
    Ok(())
}

/// iterate the palette and draw each one to screen. Ratatui comes
/// with these layouts, which ship areas to bind components to.
/// https://ratatui.rs/concepts/layout/
fn draw(frame: &mut Frame, palette: &[Rgb], name: &str) {
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
        Line::from(name).centered(),
    )
    .block(Block::default());
    frame.render_widget(title, title_area);

    // main area which displays the swatches for each color in the palette
    // split into rows of 8 with square-ish cells
    let cols_per_row = 8usize;
    let rows: Vec<&[Rgb]> = palette.chunks(cols_per_row).collect();
    let num_rows = rows.len();

    // terminal chars are ~2:1 height:width, so row height = col_width / 2 for squares
    let col_width = main_area.width as usize / cols_per_row;
    let row_height = (col_width / 2).max(1) as u16;

    let row_constraints: Vec<Constraint> = vec![Constraint::Length(row_height); num_rows];
    let row_areas = Layout::vertical(&row_constraints).split(main_area);

    for (row_idx, row_colors) in rows.iter().enumerate() {
        let col_constraints: Vec<Constraint> =
            vec![Constraint::Ratio(1, cols_per_row as u32); row_colors.len()];
        let col_areas = Layout::horizontal(&col_constraints).split(row_areas[row_idx]);

        for (col_idx, c) in row_colors.iter().enumerate() {
            let bg = Color::Rgb(c.0, c.1, c.2);
            let hex = c.hex();

            // vertically center the hex label within the swatch
            let padding = (row_height.saturating_sub(1)) / 2;
            let mut lines: Vec<Line> = vec![Line::from(""); padding as usize];
            lines.push(
                Line::from(Span::styled(hex, Style::default().fg(Color::Black).bg(bg))).centered(),
            );

            let swatch = Paragraph::new(lines)
                .style(Style::default().bg(bg))
                .block(Block::default());

            frame.render_widget(swatch, col_areas[col_idx]);
        }
    }

    // footer with information on how to leave the view (vim could never)
    let footer = Paragraph::new(Line::from("q/Esc to quit").centered())
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(footer, footer_area);
}
