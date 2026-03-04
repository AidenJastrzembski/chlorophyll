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
    let constraints: Vec<Constraint> = (0..8).map(|_| Constraint::Ratio(1, 8)).collect();
    let columns = Layout::horizontal(&constraints).split(main_area);

    for (i, c) in palette.iter().take(8).enumerate() {
        let bg = Color::Rgb(c.0, c.1, c.2);

        let hex = c.hex();
        // very similar naming conventions as web.
        // paragraph => <p>
        // the style part is like css styles and you tack on
        // methods to tack on new styles
        let swatch = Paragraph::new(
            Line::from(Span::styled(hex, Style::default().fg(Color::Black).bg(bg))).centered(),
        )
        .style(Style::default().bg(bg))
        .block(Block::default());

        frame.render_widget(swatch, columns[i]);
    }

    // footer with information on how to leave the view (vim could never)
    let footer = Paragraph::new(Line::from("q/Esc to quit").centered())
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(footer, footer_area);
}
