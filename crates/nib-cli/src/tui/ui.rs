//! Drawing. Every screen is a layout; nothing is padded by hand.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Padding, Paragraph, Wrap};

use nib_core::Summary;

use super::app::{App, MENU, Screen};

const RULE: &str = "─";

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    if area.width < 60 || area.height < 24 {
        too_small(frame, area);
        return;
    }

    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(area);

    header(frame, rows[0], app);
    rule(frame, rows[1]);
    match app.screen {
        Screen::Home => home(frame, rows[2], app),
        Screen::Add => add(frame, rows[2], app),
        Screen::Search => search(frame, rows[2], app),
        Screen::Recent => recent(frame, rows[2], app),
        Screen::Preview => preview(frame, rows[2], app),
    }
    rule(frame, rows[3]);
    footer(frame, rows[4], app);
    status(frame, rows[5], app);
}

fn too_small(frame: &mut Frame, area: Rect) {
    let text = Paragraph::new(vec![
        Line::from(Span::styled(
            "nib",
            Style::new().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!(
            "needs at least 60×24, this window is {}×{}",
            area.width, area.height
        )),
    ])
    .alignment(Alignment::Center);
    let block = Block::default().padding(Padding::new(
        0,
        0,
        area.height.saturating_div(2).saturating_sub(1),
        0,
    ));
    frame.render_widget(text.block(block), area);
}

fn header(frame: &mut Frame, area: Rect, app: &App) {
    let cols = Layout::horizontal([Constraint::Length(8), Constraint::Min(0)]).split(area);
    frame.render_widget(
        Paragraph::new(Span::styled(
            " nib",
            Style::new().fg(accent()).add_modifier(Modifier::BOLD),
        )),
        cols[0],
    );
    frame.render_widget(
        Paragraph::new(Span::styled(
            format!(
                "{} ",
                short_path(&app.store, cols[1].width.saturating_sub(1) as usize)
            ),
            muted(),
        ))
        .alignment(Alignment::Right),
        cols[1],
    );
}

fn rule(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Paragraph::new(Span::styled(RULE.repeat(area.width as usize), muted())),
        area,
    );
}

const MARK: [&str; 10] = [
    "  ▄███▄  ",
    "▄███████▄",
    "█████████",
    "█████████",
    "█████████",
    "████▀████",
    "████ ████",
    " ███ ███ ",
    "  ██ ██  ",
    "  ▀▀█▀▀  ",
];

const TAGLINE: &str = "Offline pastebin for your terminal.";

fn home(frame: &mut Frame, area: Rect, app: &App) {
    // The mark and its line are the hero; they exist only on Home, so Add and
    // the rest open as focused screens.
    let hero = (MARK.len() as u16) + 1;
    let mut y = area.y;

    if area.height >= hero + 1 + MENU.len() as u16 {
        for (i, row_text) in MARK.iter().enumerate() {
            let width = row_text.chars().count() as u16;
            let x = area.x + (area.width.saturating_sub(width)) / 2;
            let rect = Rect {
                x,
                y: y + i as u16,
                width,
                height: 1,
            };
            if i + 1 == MARK.len() {
                // The tip is the one warm pixel in the mark.
                frame.render_widget(
                    Paragraph::new(Line::from(vec![
                        Span::styled("  ▀▀", Style::new().fg(mark_colour(i))),
                        Span::styled("█", Style::new().fg(accent())),
                        Span::styled("▀▀  ", Style::new().fg(mark_colour(i))),
                    ])),
                    rect,
                );
            } else {
                frame.render_widget(
                    Paragraph::new(Span::styled(
                        row_text.to_string(),
                        Style::new().fg(mark_colour(i)),
                    )),
                    rect,
                );
            }
        }
        y += MARK.len() as u16;
        frame.render_widget(
            Paragraph::new(Span::styled(TAGLINE, muted())).alignment(Alignment::Center),
            Rect {
                y,
                height: 1,
                ..area
            },
        );
        y += 2;
    }

    let block_width = 46.min(area.width);
    let block_x = area.x + (area.width - block_width) / 2;

    for (i, (label, desc)) in MENU.iter().enumerate() {
        let selected = i == app.menu;
        let marker = if selected { "›" } else { " " };
        let label_style = if selected {
            Style::new().fg(accent()).add_modifier(Modifier::BOLD)
        } else {
            Style::new()
        };
        let line = Line::from(vec![
            Span::styled(format!(" {marker} "), Style::new().fg(accent())),
            Span::styled(format!("{label:<16}"), label_style),
            Span::styled(*desc, muted()),
        ]);
        frame.render_widget(
            Paragraph::new(line),
            Rect {
                x: block_x,
                y: y + i as u16,
                width: block_width,
                height: 1,
            },
        );
    }
    y += MENU.len() as u16;

    if area.y + area.height < y + 2 {
        return;
    }
    if area.y + area.height >= y + 5 {
        y += 1;
    }

    let count = app.recent.len();
    frame.render_widget(
        Paragraph::new(Span::styled(
            if count == 0 {
                "  recent".to_string()
            } else {
                format!("  recent · {count}")
            },
            muted(),
        )),
        Rect {
            y,
            height: 1,
            ..area
        },
    );
    y += 1;

    if app.recent.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                " nothing stored yet · n writes the first one",
                muted(),
            )),
            Rect {
                y,
                height: 1,
                ..area
            },
        );
        return;
    }

    let rows = area.y + area.height - y;
    for (i, item) in app.recent.iter().take(rows as usize).enumerate() {
        frame.render_widget(
            Paragraph::new(list_row(item, area.width)),
            Rect {
                y: y + i as u16,
                height: 1,
                ..area
            },
        );
    }
}

fn mark_colour(row: usize) -> Color {
    if no_color() {
        return Color::Reset;
    }
    const DARK: [Color; 10] = [
        Color::Rgb(215, 236, 254),
        Color::Rgb(188, 220, 247),
        Color::Rgb(166, 201, 236),
        Color::Rgb(147, 183, 221),
        Color::Rgb(130, 168, 206),
        Color::Rgb(113, 153, 191),
        Color::Rgb(98, 139, 176),
        Color::Rgb(84, 125, 162),
        Color::Rgb(72, 112, 148),
        Color::Rgb(61, 94, 128),
    ];
    const LIGHT: [Color; 10] = [
        Color::Rgb(74, 107, 140),
        Color::Rgb(70, 101, 133),
        Color::Rgb(65, 95, 126),
        Color::Rgb(60, 89, 118),
        Color::Rgb(55, 83, 111),
        Color::Rgb(50, 77, 104),
        Color::Rgb(45, 71, 96),
        Color::Rgb(40, 65, 88),
        Color::Rgb(35, 59, 80),
        Color::Rgb(30, 53, 72),
    ];
    if light_terminal() {
        LIGHT[row]
    } else {
        DARK[row]
    }
}
fn list_row(item: &Summary, width: u16) -> Line<'static> {
    let age = ago(item.created_at);
    let name_width = (width as usize).saturating_sub(2 + 4 + 2 + age.chars().count() + 2 + 1);
    let name = cut(&item.name, name_width);
    let pad = name_width.saturating_sub(name.chars().count()) + 2;
    Line::from(vec![
        Span::raw("  "),
        Span::styled(item.id.clone(), Style::new().fg(accent())),
        Span::raw("  "),
        Span::raw(name),
        Span::raw(" ".repeat(pad)),
        Span::styled(age, muted()),
        Span::raw(" "),
    ])
}

fn add(frame: &mut Frame, area: Rect, app: &App) {
    let inner = Block::default()
        .padding(Padding::new(2, 2, 1, 1))
        .inner(area);
    frame.render_widget(&app.editor, inner);
}

fn search(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::vertical([Constraint::Length(2), Constraint::Min(0)]).split(area);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("  search  ", muted()),
            Span::raw(app.query.clone()),
            Span::styled("█", Style::new().fg(accent())),
        ])),
        chunks[0],
    );

    if app.query.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled("  type to search the text", muted())),
            chunks[1],
        );
        return;
    }
    if app.hits.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled("  no matches", muted())),
            chunks[1],
        );
        return;
    }
    for (i, hit) in app.hits.iter().take(chunks[1].height as usize).enumerate() {
        let selected = i == app.list;
        let marker = if selected { "›" } else { " " };
        let style = if selected {
            Style::new().fg(accent()).add_modifier(Modifier::BOLD)
        } else {
            Style::new()
        };
        let mut spans = vec![
            Span::styled(format!(" {marker} "), Style::new().fg(accent())),
            Span::styled(hit.id.clone(), style),
            Span::raw("  "),
        ];
        spans.extend(snippet_spans(&hit.snippet, area.width));
        frame.render_widget(Paragraph::new(Line::from(spans)), row(chunks[1], i as u16));
    }
}

/// Splits the store's snippet markers into normal and highlighted spans.
fn snippet_spans(snippet: &str, width: u16) -> Vec<Span<'static>> {
    let budget = (width as usize).saturating_sub(12);
    let mut out = Vec::new();
    let mut rest = snippet;
    let mut used = 0usize;
    while used < budget {
        let Some(open) = rest.find('\u{1}') else {
            out.push(Span::styled(cut(rest, budget - used), muted()));
            break;
        };
        let before = &rest[..open];
        out.push(Span::styled(cut(before, budget - used), muted()));
        used += before.chars().count().min(budget - used);
        let after = &rest[open + 1..];
        let Some(close) = after.find('\u{2}') else {
            out.push(Span::styled(cut(after, budget - used), muted()));
            break;
        };
        let hit = &after[..close];
        if used + hit.chars().count() > budget {
            out.push(Span::styled(
                cut(hit, budget - used),
                Style::new().fg(accent()),
            ));
            break;
        }
        out.push(Span::styled(hit.to_string(), Style::new().fg(accent())));
        used += hit.chars().count();
        rest = &after[close + 1..];
    }
    out
}

fn recent(frame: &mut Frame, area: Rect, app: &App) {
    if app.recent.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled("  nothing stored yet", muted())),
            area,
        );
        return;
    }
    for (i, item) in app.recent.iter().take(area.height as usize).enumerate() {
        let selected = i == app.list;
        let marker = if selected { "›" } else { " " };
        let style = if selected {
            Style::new().fg(accent()).add_modifier(Modifier::BOLD)
        } else {
            Style::new()
        };
        let age = ago(item.created_at);
        let name_width =
            (area.width as usize).saturating_sub(2 + 4 + 2 + age.chars().count() + 2 + 1);
        let name = cut(&item.name, name_width);
        let pad = name_width.saturating_sub(name.chars().count()) + 2;
        let line = Line::from(vec![
            Span::styled(format!(" {marker} "), Style::new().fg(accent())),
            Span::styled(item.id.clone(), style),
            Span::raw("  "),
            Span::raw(name),
            Span::raw(" ".repeat(pad)),
            Span::styled(age, muted()),
            Span::raw(" "),
        ]);
        frame.render_widget(Paragraph::new(line), row(area, i as u16));
    }
}

fn preview(frame: &mut Frame, area: Rect, app: &App) {
    let Some(preview) = app.preview.as_ref() else {
        frame.render_widget(
            Paragraph::new(Span::styled("  nothing selected", muted())),
            area,
        );
        return;
    };
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .split(area);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw("  "),
            Span::styled(preview.id.clone(), Style::new().fg(accent())),
            Span::styled(
                if preview.sealed {
                    "  sealed".to_string()
                } else {
                    String::new()
                },
                muted(),
            ),
        ])),
        chunks[0],
    );
    rule(frame, chunks[1]);

    let Some(body) = preview.body.as_ref() else {
        let lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                "  sealed paste",
                Style::new().add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled("  the passphrase is never stored", muted())),
            Line::from(""),
            Line::from(vec![
                Span::styled("  passphrase  ", muted()),
                Span::raw("•".repeat(preview.pass.chars().count())),
                Span::styled("█", Style::new().fg(accent())),
            ]),
            Line::from(""),
            Line::from(match &preview.error {
                Some(error) => Span::styled(format!("  {error}"), Style::new().fg(danger())),
                None => Span::styled("  press u to unlock", muted()),
            }),
        ];
        frame.render_widget(Paragraph::new(lines), chunks[2]);
        return;
    };

    let text = Paragraph::new(body.clone())
        .wrap(Wrap { trim: false })
        .scroll((preview.scroll, 0));
    let body_area = Block::default()
        .padding(Padding::new(2, 2, 0, 0))
        .inner(chunks[2]);
    frame.render_widget(text, body_area);
}

fn footer(frame: &mut Frame, area: Rect, app: &App) {
    let keys = match app.screen {
        Screen::Home => "↑↓ move · ↵ open · n new · / find · q quit",
        Screen::Add if app.discard => "discard this paste? y discard · any other key keep",
        Screen::Add => "Ctrl+S save · Esc cancel",
        Screen::Search => "↑↓ move · ↵ open · Esc back",
        Screen::Recent => "↑↓ move · ↵ open · Esc back",
        Screen::Preview if app.preview.as_ref().is_some_and(|p| p.typing) => {
            "type passphrase · ↵ unlock · Esc cancel"
        }
        Screen::Preview => "↵ copy · ↑↓ scroll · Esc back",
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw(" "),
            Span::styled(keys, muted()),
        ])),
        area,
    );
}

fn status(frame: &mut Frame, area: Rect, app: &App) {
    if let Some((text, error)) = app.toast() {
        let style = if error {
            Style::new().fg(danger())
        } else {
            Style::new().fg(accent())
        };
        frame.render_widget(
            Paragraph::new(Line::from(vec![Span::raw("  "), Span::styled(text, style)])),
            area,
        );
        return;
    }

    let (left, right) = match app.screen {
        Screen::Home => (
            String::new(),
            format!(
                "{} {}",
                app.recent.len(),
                plural(app.recent.len(), "paste", "pastes")
            ),
        ),
        Screen::Add => (
            format!("{} chars", app.draft().chars().count()),
            "64 KB at most".to_string(),
        ),
        Screen::Search => (
            format!(
                "{} {}",
                app.hits.len(),
                plural(app.hits.len(), "hit", "hits")
            ),
            String::new(),
        ),
        Screen::Recent => (format!("{} shown", app.recent.len()), String::new()),
        Screen::Preview => (
            match app.preview.as_ref().and_then(|p| p.body.as_ref()) {
                Some(body) => format!("{} bytes", body.len()),
                None => String::new(),
            },
            String::new(),
        ),
    };

    let chunks =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(area);
    frame.render_widget(
        Paragraph::new(Span::styled(format!(" {left}"), muted())).alignment(Alignment::Left),
        chunks[0],
    );
    frame.render_widget(
        Paragraph::new(Span::styled(format!("{right} "), muted())).alignment(Alignment::Right),
        chunks[1],
    );
}

fn row(area: Rect, index: u16) -> Rect {
    Rect {
        y: area.y + index,
        height: 1,
        ..area
    }
}

fn short_path(path: &str, width: usize) -> String {
    if path.chars().count() <= width {
        return path.to_string();
    }
    let tail: String = path
        .chars()
        .rev()
        .take(width.saturating_sub(1))
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("…{tail}")
}

fn cut(text: &str, width: usize) -> String {
    text.chars().take(width).collect()
}

fn ago(created_at: i64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let seconds = ((now - created_at) / 1000).max(0);
    match seconds {
        0..=59 => "now".into(),
        60..=3599 => format!("{}m", seconds / 60),
        3600..=86_399 => format!("{}h", seconds / 3600),
        _ => format!("{}d", seconds / 86_400),
    }
}

fn plural(n: usize, one: &'static str, many: &'static str) -> &'static str {
    if n == 1 { one } else { many }
}

fn no_color() -> bool {
    std::env::var_os("NO_COLOR").is_some()
}

fn accent() -> Color {
    if no_color() {
        return Color::Reset;
    }
    if light_terminal() {
        Color::Rgb(180, 83, 31)
    } else {
        Color::Rgb(224, 122, 69)
    }
}

fn muted() -> Color {
    if no_color() {
        Color::Reset
    } else {
        Color::Indexed(245)
    }
}

fn danger() -> Color {
    if no_color() {
        Color::Reset
    } else {
        Color::Rgb(249, 112, 102)
    }
}

/// `COLORFGBG` ends in the background index; 0–8 is dark, 9–15 is light.
fn light_terminal() -> bool {
    std::env::var("COLORFGBG")
        .ok()
        .and_then(|v| v.rsplit(';').next()?.parse::<u8>().ok())
        .is_some_and(|bg| bg >= 9)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::app::App;
    use nib_core::Paste;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Buffer;

    fn render(app: &App, width: u16, height: u16) -> Buffer {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| draw(f, app)).unwrap();
        terminal.backend().buffer().clone()
    }

    fn lines(buffer: &Buffer) -> Vec<String> {
        let area = *buffer.area();
        (0..area.height)
            .map(|y| {
                (0..area.width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect()
    }

    fn app() -> App {
        let mut app = App::new("C:\\Users\\admin\\AppData\\Roaming\\nib\\nib.db".into());
        app.refresh(vec![
            Summary {
                id: "k3x9".into(),
                name: "fn parse_args() -> Args {".into(),
                size: 100,
                created_at: 0,
                enc: false,
            },
            Summary {
                id: "r8ce".into(),
                name: "deploy key".into(),
                size: 40,
                created_at: 0,
                enc: true,
            },
        ]);
        app
    }

    #[test]
    fn home_opens_on_the_mark_and_its_tagline() {
        let text = lines(&render(&app(), 80, 24)).join("\n");
        assert!(text.contains("▄███▄"));
        assert!(text.contains("Offline pastebin for your terminal."));
        let tagline = text
            .lines()
            .find(|l| l.contains("Offline pastebin"))
            .unwrap();
        let pad = tagline.find('O').unwrap();
        assert!(pad > 10, "the tagline should be centred, got {pad}");
    }

    #[test]
    fn the_mark_leaves_the_screen_when_focus_moves_to_add() {
        let mut app = app();
        app.handle(ratatui::crossterm::event::KeyEvent::from(
            ratatui::crossterm::event::KeyCode::Enter,
        ));
        let text = lines(&render(&app, 80, 24)).join("\n");
        assert!(!text.contains("Offline pastebin"));
        assert!(!text.contains("▄███▄"));
        assert!(text.contains("Ctrl+S save"));
    }

    #[test]
    fn home_still_fits_at_the_minimum_height() {
        let text = lines(&render(&app(), 60, 24)).join("\n");
        assert!(text.contains("Offline pastebin"));
        assert!(text.contains("Add a paste"));
        assert!(text.contains("k3x9"));
    }

    #[test]
    fn home_shows_the_menu_and_a_selection_marker() {
        let text = lines(&render(&app(), 100, 30)).join("\n");
        assert!(text.contains("Add a paste"));
        assert!(text.contains("Find a paste"));
        assert!(text.contains("Recent pastes"));
        assert!(text.contains("› Add a paste"));
        assert!(text.contains("k3x9"));
    }

    #[test]
    fn no_menu_line_collides_at_the_minimum_width() {
        for line in lines(&render(&app(), 60, 24)) {
            for label in ["Add a paste", "Find a paste", "Recent pastes"] {
                if let Some(at) = line.find(label) {
                    let after = &line[at + label.len()..];
                    let spaces = after.chars().take_while(|c| *c == ' ').count();
                    assert!(spaces >= 2, "no gap after {label:?}: {line:?}");
                }
            }
        }
    }

    #[test]
    fn the_selected_row_moves_with_the_menu() {
        let mut app = app();
        app.menu = 2;
        let lines = lines(&render(&app, 100, 30));
        let add = lines.iter().find(|l| l.contains("Add a paste")).unwrap();
        let recent = lines.iter().find(|l| l.contains("Recent pastes")).unwrap();
        assert!(!add.contains('›'));
        assert!(recent.contains('›'));
    }

    #[test]
    fn every_footer_names_only_the_keys_for_that_screen() {
        let home = lines(&render(&app(), 100, 30)).join("\n");
        assert!(home.contains("q quit"));

        let mut app = app();
        app.handle(ratatui::crossterm::event::KeyEvent::from(
            ratatui::crossterm::event::KeyCode::Enter,
        ));
        let add = lines(&render(&app, 100, 30)).join("\n");
        assert!(add.contains("Ctrl+S save"));
        assert!(add.contains("chars"));
        assert!(!add.contains("q quit"));
    }

    #[test]
    fn a_small_terminal_draws_only_the_size_message() {
        let text = lines(&render(&app(), 59, 23)).join("\n");
        assert!(text.contains("needs at least 60×24"));
        assert!(!text.contains("Add a paste"));
    }

    #[test]
    fn a_sealed_preview_never_prints_the_envelope() {
        let mut app = app();
        app.open_preview("r8ce".into(), Screen::Home);
        app.loaded(Paste {
            id: "r8ce".into(),
            body: "nibenc1:zzz".into(),
            enc: true,
            label: None,
        });
        let text = lines(&render(&app, 100, 30)).join("\n");
        assert!(text.contains("sealed paste"));
        assert!(text.contains("passphrase"));
        assert!(!text.contains("nibenc1"));
    }

    #[test]
    fn the_store_path_is_shortened_not_cut_in_the_middle() {
        assert_eq!(short_path("C:\\a\\b\\c", 20), "C:\\a\\b\\c");
        let short = short_path("C:\\very\\long\\path\\to\\nib.db", 12);
        assert!(short.starts_with('…'));
        assert!(short.ends_with("nib.db"));
        assert_eq!(short.chars().count(), 12);
    }
}
