use crate::config::ThemeConfig;
use crate::theme::{Theme, list_wallpapers};
use crate::utils::PaletteSize;
use crate::utils::cache;
use crate::utils::colorspace::Rgb;
use crate::utils::palette::{self, LabeledColors};
use std::path::Path;
use anyhow::{Ok, Result};
use std::collections::HashMap;
use ratatui::{
    Frame,
    crossterm::event::{self, Event, KeyCode, KeyModifiers},
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph},
};

// a wallpaper and its cached palette if one exists
struct WallpaperEntry {
    name: String,
    palette: Option<(Vec<Rgb>, LabeledColors)>,
    is_custom_theme: bool,
}

// holds all the state for the list tui, filtering, and selection
struct ListApp {
    wallpapers: Vec<WallpaperEntry>,
    // indices into wallpapers vec that match the current search query
    filtered_indices: Vec<usize>,
    list_state: ListState,
    search_query: String,
}

impl ListApp {
    fn new(wallpapers: Vec<WallpaperEntry>) -> Self {
        let filtered_indices: Vec<usize> = (0..wallpapers.len()).collect();
        let mut list_state = ListState::default();
        if !filtered_indices.is_empty() {
            list_state.select(Some(0));
        }
        Self {
            wallpapers,
            filtered_indices,
            list_state,
            search_query: String::new(),
        }
    }

    /// used when query changes
    fn refilter(&mut self) {
        let query = self.search_query.to_lowercase();
        self.filtered_indices = self
            .wallpapers
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.name.to_lowercase().contains(&query))
            .map(|(i, _)| i)
            .collect();

        if self.filtered_indices.is_empty() {
            self.list_state.select(None);
        } else {
            // clamp selection to valid range
            let sel = self.list_state.selected().unwrap_or(0);
            let clamped = sel.min(self.filtered_indices.len() - 1);
            self.list_state.select(Some(clamped));
        }
    }

    // wrapping navigation so you can scroll past the end back to the top
    fn move_down(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => (i + 1) % self.filtered_indices.len(),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn move_up(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(0) | None => self.filtered_indices.len() - 1,
            Some(i) => i - 1,
        };
        self.list_state.select(Some(i));
    }

    fn selected_entry(&self) -> Option<&WallpaperEntry> {
        let sel = self.list_state.selected()?;
        let idx = *self.filtered_indices.get(sel)?;
        Some(&self.wallpapers[idx])
    }

    fn selected_name(&self) -> Option<String> {
        self.selected_entry().map(|e| e.name.clone())
    }
}

fn load_cached_palette(
    path: &Path,
    palette_size: PaletteSize,
) -> Option<(Vec<Rgb>, LabeledColors)> {
    let hash = Theme::new(path.to_path_buf()).hash(palette_size).ok()?;
    let colors = cache::load_cache(&hash).ok().flatten()?;
    let labels = palette::assign_labels(&colors);
    Some((colors, labels))
}

/// interactive list search tui
/// previews wallpapers in wallpaper dir and cached previews if available
pub fn list_themes(
    wallpaper_dir: &str,
    palette_size: PaletteSize,
    custom_themes: &HashMap<String, ThemeConfig>,
) -> Result<Option<String>> {
    let paths = list_wallpapers(wallpaper_dir)?;
    if paths.is_empty() && custom_themes.is_empty() {
        println!("No wallpapers found in {wallpaper_dir}");
        return Ok(None);
    }

    // only load palettes from cache here, don't extract new ones
    // since that would be slow for a big wallpaper dir
    let mut wallpapers: Vec<WallpaperEntry> = paths
        .into_iter()
        .map(|path| {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("?")
                .to_string();

            let palette = load_cached_palette(&path, palette_size);

            WallpaperEntry {
                name,
                palette,
                is_custom_theme: false,
            }
        })
        .collect();

    // append custom themes from config
    let mut theme_names: Vec<&String> = custom_themes.keys().collect();
    theme_names.sort();
    wallpapers.extend(theme_names.into_iter().map(|name| {
        let tc = &custom_themes[name];
        let palette = load_cached_palette(Path::new(&tc.path), palette_size);
        WallpaperEntry {
            name: name.clone(),
            palette,
            is_custom_theme: true,
        }
    }));

    let mut app = ListApp::new(wallpapers);
    let mut selected_name: Option<String> = None;

    ratatui::run(|terminal| {
        loop {
            terminal.draw(|frame| draw(frame, &mut app))?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => break,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    KeyCode::Up => app.move_up(),
                    KeyCode::Down => app.move_down(),
                    KeyCode::Enter => {
                        selected_name = app.selected_name();
                        break;
                    }
                    KeyCode::Backspace => {
                        app.search_query.pop();
                        app.refilter();
                    }
                    KeyCode::Char(c) => {
                        app.search_query.push(c);
                        app.refilter();
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    })?;

    Ok(selected_name)
}

// layout is a header bar, then a left/right split (list + preview), then a search bar
fn draw(frame: &mut Frame, app: &mut ListApp) {
    let [header_area, middle_area, search_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    // 35/65 split gives the palette swatches enough room to breathe
    let [list_area, preview_area] =
        Layout::horizontal([Constraint::Percentage(35), Constraint::Percentage(65)])
            .areas(middle_area);

    draw_header(frame, header_area);
    draw_list(frame, app, list_area);
    draw_preview(frame, app, preview_area);
    draw_search(frame, app, search_area);
}

fn draw_header(frame: &mut Frame, area: Rect) {
    let header = Paragraph::new(Line::from("  ↑/↓: navigate  Enter: apply  Esc: quit").centered())
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(header, area);
}

fn draw_list(frame: &mut Frame, app: &mut ListApp, area: Rect) {
    // wallpapers without a cached palette are grayed out so the user
    // knows they need to run `chlorophyll cache <name>` first
    let items: Vec<ListItem> = app
        .filtered_indices
        .iter()
        .map(|&idx| {
            let entry = &app.wallpapers[idx];
            let style = if entry.palette.is_some() {
                Style::default()
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let mut spans = vec![Span::styled(&entry.name, style)];
            if entry.is_custom_theme {
                spans.push(Span::styled(" [theme]", Style::default().fg(Color::Cyan)));
            }
            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items)
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn draw_search(frame: &mut Frame, app: &ListApp, area: Rect) {
    let search_line = Line::from(vec![
        Span::styled("> ", Style::default().fg(Color::Cyan)),
        Span::raw(&app.search_query),
        Span::styled("_", Style::default().fg(Color::DarkGray)),
    ]);
    let search = Paragraph::new(search_line);
    frame.render_widget(search, area);
}

// shows the palette swatches for the selected wallpaper, or a placeholder if uncached
fn draw_preview(frame: &mut Frame, app: &ListApp, area: Rect) {
    if let Some(entry) = app.selected_entry()
        && let Some((palette, labels)) = &entry.palette
    {
        let [title_area, swatches_area] =
            Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).areas(area);

        let title = Paragraph::new(Line::from(&*entry.name).centered());
        frame.render_widget(title, title_area);

        draw_swatches(frame, palette, labels, swatches_area);
        return;
    }

    let msg = Paragraph::new(Line::from("No cached palette").centered())
        .style(Style::default().fg(Color::DarkGray));
    let [_, center, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(area);
    frame.render_widget(msg, center);
}

// renders the color grid, same logic as preview.rs but adapted
// for the smaller area inside the list view
fn draw_swatches(frame: &mut Frame, palette: &[Rgb], labels: &LabeledColors, area: Rect) {
    let cols_per_row = 8usize;
    let num_rows = palette.len().div_ceil(cols_per_row);

    // terminal chars are ~2:1 height:width, so halve the width for square-ish cells
    let col_width = area.width as usize / cols_per_row;
    let row_height = (col_width / 2).max(1) as u16;

    let row_constraints: Vec<Constraint> = vec![Constraint::Length(row_height); num_rows];
    let row_areas = Layout::vertical(&row_constraints).split(area);

    for (row_idx, row_colors) in palette.chunks(cols_per_row).enumerate() {
        let col_constraints: Vec<Constraint> =
            vec![Constraint::Ratio(1, cols_per_row as u32); row_colors.len()];
        let col_areas = Layout::horizontal(&col_constraints).split(row_areas[row_idx]);

        for (col_idx, c) in row_colors.iter().enumerate() {
            let bg = Color::Rgb(c.0, c.1, c.2);
            let hex = c.hex();
            let label = labels.label_for(c);

            // vertically center the hex + optional label within the swatch
            let content_lines: u16 = if label.is_some() { 2 } else { 1 };
            let padding = (row_height.saturating_sub(content_lines)) / 2;
            let mut lines: Vec<Line> = vec![Line::from(""); padding as usize];
            lines.push(
                Line::from(Span::styled(hex, Style::default().fg(Color::Black).bg(bg))).centered(),
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

            let swatch = Paragraph::new(lines).style(Style::default().bg(bg));

            frame.render_widget(swatch, col_areas[col_idx]);
        }
    }
}
