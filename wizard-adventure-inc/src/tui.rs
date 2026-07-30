//! Ratatui frontend. All game rules live in [`crate::game`]; this module only
//! renders state and translates key presses into engine calls.

use std::collections::HashSet;
use std::time::{Duration, Instant};

use anyhow::Result;
use ratatui::{
    Frame,
    crossterm::event::{self, Event as TermEvent, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::game::{Event as GameEvent, Game, NodeStatus};

struct App {
    game: Game,
    /// Rows of the action tree as (node id, depth), rebuilt every frame.
    rows: Vec<(u32, usize)>,
    list_state: ListState,
    log: Vec<String>,
    flash: Option<String>,
}

impl App {
    fn new(game: Game) -> App {
        let mut app = App {
            game,
            rows: Vec::new(),
            list_state: ListState::default().with_selected(Some(0)),
            log: Vec::new(),
            flash: None,
        };
        app.rebuild_rows();
        app
    }

    /// Depth-first walk of the discovered part of the graph, so the list reads
    /// as a tree growing out of the start node.
    fn rebuild_rows(&mut self) {
        fn walk(game: &Game, id: u32, depth: usize, seen: &mut HashSet<u32>, out: &mut Vec<(u32, usize)>) {
            if !seen.insert(id) {
                return;
            }
            out.push((id, depth));
            if let Some(node) = game.node(id) {
                for &child in &node.connects_to {
                    if game.is_discovered(child) {
                        walk(game, child, depth + 1, seen, out);
                    }
                }
            }
        }
        self.rows.clear();
        let mut seen = HashSet::new();
        walk(&self.game, self.game.start_id(), 0, &mut seen, &mut self.rows);

        let max = self.rows.len().saturating_sub(1);
        let selected = self.list_state.selected().unwrap_or(0).min(max);
        self.list_state.select(Some(selected));
    }

    fn selected_id(&self) -> Option<u32> {
        self.rows.get(self.list_state.selected()?).map(|(id, _)| *id)
    }

    fn apply_events(&mut self, events: Vec<GameEvent>) {
        for event in events {
            match event {
                GameEvent::Log { text, .. } => {
                    self.log.extend(text.lines().map(String::from));
                }
                GameEvent::ItemAdded { item, quantity } => {
                    self.log.push(format!("[+] {} added to inventory", fmt_item(&item, quantity)));
                }
                GameEvent::ItemRemoved { item, quantity } => {
                    self.log.push(format!("[-] {} removed from inventory", fmt_item(&item, quantity)));
                }
                GameEvent::Discovered(_) => self.rebuild_rows(),
                GameEvent::StateChanged(state) => {
                    if state == crate::game::STATE_OUT_OF_TIME {
                        self.log.push("You ran out of time.".into());
                    }
                }
            }
        }
    }
}

fn fmt_item(item: &str, quantity: u32) -> String {
    if quantity == 1 {
        item.to_string()
    } else {
        format!("{item} ×{quantity}")
    }
}

pub fn run(raw: &str) -> Result<()> {
    let mut terminal = ratatui::init();
    let result = event_loop(&mut terminal, raw);
    ratatui::restore();
    result
}

fn event_loop(terminal: &mut ratatui::DefaultTerminal, raw: &str) -> Result<()> {
    let mut app = App::new(Game::from_rec_str(raw)?);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|frame| draw(frame, &mut app))?;

        if event::poll(Duration::from_millis(50))? {
            if let TermEvent::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.flash = None;
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char('r') => {
                            app = App::new(Game::from_rec_str(raw)?);
                            last_tick = Instant::now();
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.list_state.select_previous();
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            let max = app.rows.len().saturating_sub(1);
                            let next = app.list_state.selected().unwrap_or(0).saturating_add(1);
                            app.list_state.select(Some(next.min(max)));
                        }
                        KeyCode::Enter => {
                            if let Some(id) = app.selected_id() {
                                if let Err(err) = app.game.start_action(id) {
                                    app.flash = Some(err.to_string());
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        let dt = last_tick.elapsed().as_secs_f64();
        last_tick = Instant::now();
        let events = app.game.tick(dt);
        app.apply_events(events);
    }
}

fn draw(frame: &mut Frame, app: &mut App) {
    let [header, main, log_area, status] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(8),
        Constraint::Percentage(30),
        Constraint::Length(1),
    ])
    .areas(frame.area());
    let [tree_area, detail_area] =
        Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)]).areas(main);

    draw_header(frame, app, header);
    draw_tree(frame, app, tree_area);
    draw_detail(frame, app, detail_area);
    draw_log(frame, app, log_area);
    draw_status(frame, app, status);

    if app.game.is_over() {
        draw_end_overlay(frame, app);
    }
}

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let game = &app.game;
    let time = if game.max_time().is_finite() {
        format!("Time: {:.0}/{:.0}s", game.time_used(), game.max_time())
    } else {
        format!("Time: {:.0}s", game.time_used())
    };
    let inventory = if game.inventory().is_empty() {
        "empty".to_string()
    } else {
        game.inventory()
            .iter()
            .map(|(item, &quantity)| fmt_item(item, quantity))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let line = Line::from(vec![
        Span::styled(time, Style::new().fg(Color::Yellow)),
        Span::raw("  |  Inventory: "),
        Span::styled(inventory, Style::new().fg(Color::Cyan)),
    ]);
    frame.render_widget(
        Paragraph::new(line).block(Block::default().borders(Borders::ALL).title(" Wizard Adventure ")),
        area,
    );
}

fn draw_tree(frame: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .rows
        .iter()
        .map(|&(id, depth)| {
            let node = app.game.node(id).expect("row ids are valid");
            let indent = "  ".repeat(depth);
            let (marker, style) = match app.game.node_status(id) {
                NodeStatus::Completed => ("✓ ", Style::new().fg(Color::DarkGray)),
                NodeStatus::InProgress => ("… ", Style::new().fg(Color::Yellow)),
                NodeStatus::Locked => ("✗ ", Style::new().fg(Color::Red)),
                NodeStatus::Available => ("• ", Style::new().fg(Color::White)),
            };
            let mut spans = vec![Span::raw(indent), Span::styled(marker, style), Span::styled(&node.title, style)];
            if let NodeStatus::Locked = app.game.node_status(id) {
                spans.push(Span::styled(
                    format!("  (needs: {})", app.game.missing_requirements(id).join(", ")),
                    Style::new().fg(Color::Red).add_modifier(Modifier::DIM),
                ));
            }
            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Actions "))
        .highlight_style(Style::new().bg(Color::Blue).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");
    frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn draw_detail(frame: &mut Frame, app: &App, area: Rect) {
    let Some(id) = app.selected_id() else {
        return;
    };
    let node = app.game.node(id).expect("selected id is valid");

    let in_progress = app.game.active_action().filter(|&(active, _)| active == id);
    let [text_area, gauge_area] =
        Layout::vertical([Constraint::Min(3), Constraint::Length(3)]).areas(area);

    let description = node.description.as_deref().unwrap_or("");
    let text = Paragraph::new(description)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", node.title))
                .title_style(Style::new().bold()),
        );
    frame.render_widget(text, text_area);

    if let Some((_, progress)) = in_progress {
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL))
            .gauge_style(Style::new().fg(Color::Yellow))
            .ratio(progress)
            .label(format!("working… {:.0}%", progress * 100.0));
        frame.render_widget(gauge, gauge_area);
    } else {
        let hint = format!("takes {:.0}s — press Enter to do it", node.time_to_complete);
        frame.render_widget(
            Paragraph::new(hint.dim()).block(Block::default().borders(Borders::ALL)),
            gauge_area,
        );
    }
}

fn draw_log(frame: &mut Frame, app: &App, area: Rect) {
    let inner_height = area.height.saturating_sub(2) as usize;
    let scroll = app.log.len().saturating_sub(inner_height) as u16;
    let text: Vec<Line> = app.log.iter().map(|l| Line::raw(l.as_str())).collect();
    let log = Paragraph::new(text)
        .scroll((scroll, 0))
        .block(Block::default().borders(Borders::ALL).title(" Log "));
    frame.render_widget(log, area);
}

fn draw_status(frame: &mut Frame, app: &App, area: Rect) {
    let line = match &app.flash {
        Some(msg) => Line::from(Span::styled(msg.clone(), Style::new().fg(Color::Red))),
        None => Line::from("↑/↓ select · Enter act · r restart · q quit".dim()),
    };
    frame.render_widget(Paragraph::new(line), area);
}

fn draw_end_overlay(frame: &mut Frame, app: &App) {
    let (title, color) = match app.game.state() {
        "good-ending" => ("THE END", Color::Green),
        "bad-ending" => ("GAME OVER", Color::Red),
        crate::game::STATE_OUT_OF_TIME => ("OUT OF TIME", Color::Red),
        other => (other, Color::Magenta),
    };
    let lines = vec![
        Line::from(Span::styled(title, Style::new().fg(color).bold())).centered(),
        Line::raw(""),
        Line::from("press r to play again, q to quit").centered(),
    ];

    let area = centered_rect(40, 5, frame.area());
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).border_style(Style::new().fg(color))),
        area,
    );
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let [_, mid, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(height),
        Constraint::Fill(1),
    ])
    .areas(area);
    let [_, rect, _] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(width),
        Constraint::Fill(1),
    ])
    .areas(mid);
    rect
}
