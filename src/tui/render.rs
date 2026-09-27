//! Rendering for the TUI.
//!
//! Everything here is pure: it reads the [`crate::tui::App`] state and draws widgets. There is
//! no I/O beyond writing to the ratatui frame buffer.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::core::bank::Bank;
use crate::core::board::Board;
use crate::core::chains::{HotelChain, PriceLevel};
use crate::core::chains_mgr::HotelChainManager;
use crate::core::game::{FinalResult, InputRequest};
use crate::tui::{App, DemoState, Overlay, Screen};

/// The minimum terminal size required for the play layout.
pub const MIN_WIDTH: u16 = 100;
pub const MIN_HEIGHT: u16 = 24;

/// Maps a core [`ChainColor`](crate::core::chains::ChainColor) to a ratatui color.
fn chain_color(chain: &HotelChain) -> Color {
    let c = chain.color();
    Color::Rgb(c.r, c.g, c.b)
}

fn price_color(level: PriceLevel) -> Color {
    match level {
        PriceLevel::Low => Color::Red,
        PriceLevel::Medium => Color::Yellow,
        PriceLevel::High => Color::Green,
    }
}

/// Entry point: draws whatever the app is currently showing.
pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.size();
    if size.width < MIN_WIDTH || size.height < MIN_HEIGHT {
        render_too_small(frame, size);
        return;
    }
    match app.screen {
        Screen::Setup => render_setup(frame, app, size),
        Screen::Play => render_play(frame, app, size),
        Screen::GameOver => render_gameover(frame, app, size),
        Screen::Demo => render_demo(frame, app, size),
        Screen::Lan => render_lan(frame, app, size),
    }
    // Overlays are drawn last so they sit on top.
    if let Some(overlay) = app.overlay {
        match overlay {
            Overlay::InfoCard => render_infocard(frame, size),
            Overlay::Help => render_help(frame, size),
        }
    }
}

/// Overlay shown when the terminal is too small.
fn render_too_small(frame: &mut Frame, size: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("Acquire");
    let text = Paragraph::new(vec![
        Line::from(Span::styled(
            "Terminal too small.\n",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(format!(
            "Need at least {MIN_WIDTH}×{MIN_HEIGHT}; you have {}×{}.",
            size.width, size.height
        )),
        Line::from("Please resize your terminal."),
    ])
    .alignment(Alignment::Center);
    frame.render_widget(block, size);
    frame.render_widget(text, size.inner(&ratatui::layout::Margin::new(2, 2)));
}

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

fn render_setup(frame: &mut Frame, app: &App, size: Rect) {
    let outer = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("Acquire — New Game");

    let players_marker = if app.setup_field == crate::tui::SetupField::Players {
        ">"
    } else {
        " "
    };
    let name_marker = if app.setup_field == crate::tui::SetupField::Name {
        ">"
    } else {
        " "
    };
    let name_display = if app.human_name.is_empty() {
        "You (default)".to_string()
    } else {
        app.human_name.clone()
    };

    let body = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Play single-player against bots.",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("  {players_marker} "),
                Style::default().fg(Color::Cyan),
            ),
            Span::from(format!(
                "Players ....... {}  (1 human + {} bots)",
                app.player_count,
                app.player_count - 1
            )),
        ]),
        Line::from(vec![
            Span::styled(
                format!("  {name_marker} "),
                Style::default().fg(Color::Cyan),
            ),
            Span::from(format!("Your name ..... {}", name_display)),
        ]),
        Line::from(format!(
            "  Hide extra info {}   Skip dialogues {}   Small board {}",
            yes_no(app.settings.hide_extra_info),
            yes_no(app.settings.skip_dialogues),
            yes_no(app.settings.small_board)
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Keys:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  Tab/↑/↓ ... switch field"),
        Line::from("  2-6 ....... set number of players"),
        Line::from("  (type) .... set your name when Name field is selected"),
        Line::from("  Enter ..... start the game"),
        Line::from("  h / i ..... help / stock info card"),
        Line::from("  q ......... quit"),
    ];
    let text = Paragraph::new(body).block(outer);
    frame.render_widget(text, size);
}

fn yes_no(b: bool) -> &'static str {
    if b {
        "on"
    } else {
        "off"
    }
}

// ---------------------------------------------------------------------------
// Play
// ---------------------------------------------------------------------------

fn render_play(frame: &mut Frame, app: &App, size: Rect) {
    let title = match &app.pending {
        Some((player, _)) => {
            let name = app
                .game
                .as_ref()
                .map(|g| g.players[*player].name.clone())
                .unwrap_or_default();
            format!("Acquire · {}'s turn", name)
        }
        None => format!(
            "Acquire · Round {}",
            app.game.as_ref().map(|g| g.round_number).unwrap_or(0)
        ),
    };

    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            if app.show_log {
                Constraint::Length(8)
            } else {
                Constraint::Length(0)
            },
        ])
        .split(size);

    // Title bar.
    let title_block = Block::default().borders(Borders::NONE).title(Span::styled(
        title,
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ));
    let title_para = Paragraph::new(" ").block(title_block);
    frame.render_widget(title_para, root[0]);

    // Middle: board on the left, panel + table on the right.
    let middle = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(root[1]);
    render_board(frame, app, middle[0]);
    render_right(frame, app, middle[1]);

    // Log.
    if app.show_log {
        render_log(frame, app, root[2]);
    }

    // Pending dialog on top of everything.
    if let Some((player, request)) = &app.pending {
        render_dialog(frame, app, player, request, size);
    }
}

/// Renders the 9×12 board with colored chains and the selected-card highlight.
fn render_board(frame: &mut Frame, app: &App, area: Rect) {
    let game = match &app.game {
        Some(g) => g,
        None => return,
    };
    let board: &Board = &game.board;
    let _chains: &HotelChainManager = &game.chains;

    // Determine which positions are highlighted by the selected hand card.
    let mut highlight_origin: Option<crate::core::board::Position> = None;
    let mut highlight_set: Vec<crate::core::board::Position> = Vec::new();
    if let Some((player, InputRequest::ChooseCard { legal })) = &app.pending {
        if let Some(sel) = legal.get(app.selected_card) {
            highlight_origin = Some(*sel);
            if let Some(card) = game.players[*player]
                .analyzed_cards
                .iter()
                .find(|c| c.position == *sel)
            {
                match &card.place_hotel_case {
                    crate::core::rules::PlaceHotelCase::NewChain(positions)
                    | crate::core::rules::PlaceHotelCase::ExtendsChain(_, positions) => {
                        highlight_set = positions.clone();
                    }
                    crate::core::rules::PlaceHotelCase::Fusion(fusion_chains, _origin) => {
                        for c in fusion_chains {
                            highlight_set.extend(game.chains.chain_positions(c));
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    let small = app.settings.small_board;

    // Build the board as a Table so ratatui guarantees column alignment.
    // Column 0 = row label (A–I); columns 1..=12 = the 12 cells.
    let mut rows: Vec<Row> = Vec::new();
    for (r, row) in board.pieces.iter().enumerate() {
        let letter = crate::core::board::letter::LETTERS[r];
        let mut cells = vec![
            Cell::from(letter.to_string()).style(Style::default().add_modifier(Modifier::BOLD))
        ];
        for piece in row {
            cells.push(board_cell(piece, &highlight_origin, &highlight_set));
        }
        rows.push(Row::new(cells));
    }

    // First (label) column is narrow; every cell column is a fixed width so empty
    // and filled cells always line up.
    let cell_w = if small { 2 } else { 3 };
    let mut widths: Vec<Constraint> = vec![Constraint::Length(2)];
    for _ in 0..12 {
        widths.push(Constraint::Length(cell_w));
    }

    let table = Table::new(rows, widths).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Board"),
    );
    frame.render_widget(table, area);
}

/// Builds the styled cell for a single board position.
fn board_cell(
    piece: &crate::core::board::Piece,
    highlight_origin: &Option<crate::core::board::Position>,
    highlight_set: &[crate::core::board::Position],
) -> Cell<'static> {
    let (content, mut style): (String, Style) = if piece.piece_set {
        match piece.chain {
            Some(chain) => (
                chain.identifier().to_string(),
                Style::default().fg(chain_color(&chain)),
            ),
            None => ("X".to_string(), Style::default().fg(Color::White)),
        }
    } else {
        (String::new(), Style::default().fg(Color::DarkGray))
    };

    // Highlight the selected card's origin and the positions it would found/extend.
    if let Some(origin) = highlight_origin {
        if *origin == piece.position {
            style = style.add_modifier(Modifier::REVERSED);
        } else if highlight_set.contains(&piece.position) && piece.piece_set {
            style = style.add_modifier(Modifier::UNDERLINED);
        }
    }

    let text = Text::from(Line::from(content).alignment(Alignment::Center));
    Cell::new(text).style(style)
}

/// The right-hand column: player panel on top, chain table below.
fn render_right(frame: &mut Frame, app: &App, area: Rect) {
    let _game = match &app.game {
        Some(g) => g,
        None => return,
    };
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(6)])
        .split(area);
    render_panel(frame, app, layout[0]);
    render_table(frame, app, layout[1]);
}

/// The human player's money, hand and owned stocks.
fn render_panel(frame: &mut Frame, app: &App, area: Rect) {
    let game = match &app.game {
        Some(g) => g,
        None => return,
    };
    let human = match game.players.iter().find(|p| p.name == app.human_name) {
        Some(h) => h,
        None => return,
    };

    let mut lines = vec![Line::from(Span::styled(
        format!("{} (id {})", human.name, human.id),
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::Cyan),
    ))];
    lines.push(Line::from(format!("  Money: {:>8} €", human.money)));
    // Hand (show case description).
    let hand: Vec<String> = human
        .analyzed_cards
        .iter()
        .map(|c| {
            let desc = match &c.place_hotel_case {
                crate::core::rules::PlaceHotelCase::SingleHotel => "single".to_string(),
                crate::core::rules::PlaceHotelCase::NewChain(_) => "new chain".to_string(),
                crate::core::rules::PlaceHotelCase::ExtendsChain(chain, pos) => {
                    format!("extend {} +{}", chain, pos.len())
                }
                crate::core::rules::PlaceHotelCase::Fusion(chains, _) => {
                    format!("fuse {} chains", chains.len())
                }
                crate::core::rules::PlaceHotelCase::Illegal(reason) => {
                    format!("illegal: {}", reason.reason())
                }
            };
            format!("  {}  [{}]", c.position, desc)
        })
        .collect();
    lines.push(Line::from("  Hand:"));
    for h in hand.iter().take(4) {
        lines.push(Line::from(h.as_str()));
    }
    if hand.is_empty() {
        lines.push(Line::from("   (empty)"));
    }
    // Owned stocks.
    let mut stocks = Vec::new();
    for chain in HotelChain::iterator() {
        let n = *human.owned_stocks.stocks_for_hotel(chain);
        if n > 0 {
            stocks.push(format!("{} {}", chain, n));
        }
    }
    lines.push(Line::from(format!(
        "  Stocks: {}",
        if stocks.is_empty() {
            "none".to_string()
        } else {
            stocks.join(" · ")
        }
    )));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("You");
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

/// The chain table: name, level, length, price, bank/own stocks, bonuses, stars.
fn render_table(frame: &mut Frame, app: &App, area: Rect) {
    let game = match &app.game {
        Some(g) => g,
        None => return,
    };
    let chains = &game.chains;
    let bank = &game.bank;
    let human = match game.players.iter().find(|p| p.name == app.human_name) {
        Some(h) => h,
        None => return,
    };
    let hide = app.settings.hide_extra_info;

    let header = Row::new(vec![
        Cell::from("Chain"),
        Cell::from("Lvl"),
        Cell::from("Len"),
        Cell::from("Range"),
        Cell::from("Bank"),
        Cell::from("Own"),
        Cell::from("Price"),
        Cell::from("10×"),
        Cell::from("5×"),
        Cell::from(""),
    ]);

    let mut rows: Vec<Row> = Vec::new();
    for chain in HotelChain::iterator() {
        let active = chains.chain_status(chain);
        let base = if active {
            Style::default()
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let name_style = if active {
            base.fg(chain_color(chain))
        } else {
            base
        };
        let level = chain.price_level();
        let level_txt = match level {
            PriceLevel::Low => "L",
            PriceLevel::Medium => "M",
            PriceLevel::High => "H",
        };
        let level_cell = Cell::from(level_txt).style(
            Style::default()
                .fg(if active {
                    price_color(level)
                } else {
                    Color::DarkGray
                })
                .add_modifier(Modifier::BOLD),
        );
        let len = chains.chain_length(chain);
        let price = if active {
            Bank::stock_price(chains, chain)
        } else {
            0
        };
        let bank_n = if active {
            *bank.stocks_available(chain, chains)
        } else {
            0
        };
        let own_n = *human.owned_stocks.stocks_for_hotel(chain);
        let star = if hide || !active {
            " "
        } else if bank.is_largest_shareholder(human.id, chain) {
            "★"
        } else if bank.is_second_largest_shareholder(human.id, chain) {
            "☆"
        } else {
            " "
        };
        rows.push(Row::new(vec![
            Cell::from(chain.name()).style(name_style),
            level_cell,
            Cell::from(len.to_string()).style(base),
            Cell::from(chains.price_range(chain).trim().to_string()).style(base),
            Cell::from(bank_n.to_string()).style(base),
            Cell::from(own_n.to_string()).style(base),
            Cell::from(format!("{price} €")).style(base),
            Cell::from(format!("{0} €", price * 10)).style(base),
            Cell::from(format!("{0} €", price * 5)).style(base),
            Cell::from(star).style(Style::default().fg(if star == "★" {
                Color::Yellow
            } else if star == "☆" {
                Color::Gray
            } else {
                Color::Reset
            })),
        ]));
    }

    let widths = [
        Constraint::Length(11),
        Constraint::Length(4),
        Constraint::Length(4),
        Constraint::Length(7),
        Constraint::Length(5),
        Constraint::Length(4),
        Constraint::Length(7),
        Constraint::Length(7),
        Constraint::Length(7),
        Constraint::Length(3),
    ];
    let table = Table::new(rows, widths).header(header).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Chains"),
    );
    frame.render_widget(table, area);
}

/// The scrolling event log.
fn render_log(frame: &mut Frame, app: &App, area: Rect) {
    let visible: Vec<&String> = {
        let total = app.log_lines.len();
        if total == 0 {
            Vec::new()
        } else {
            let window = area.height.saturating_sub(2) as usize;
            let window = window.max(1);
            let scroll = app.log_scroll.min(total.saturating_sub(1));
            let end = total - scroll;
            let start = end.saturating_sub(window);
            app.log_lines[start..end].iter().collect()
        }
    };
    let lines: Vec<Line> = visible.iter().map(|l| Line::from(l.as_str())).collect();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("Log");
    frame.render_widget(
        Paragraph::new(lines).block(block).wrap({
            use ratatui::widgets::Wrap;
            Wrap { trim: false }
        }),
        area,
    );
}

// ---------------------------------------------------------------------------
// Dialog for the pending request
// ---------------------------------------------------------------------------

fn render_dialog(frame: &mut Frame, app: &App, player: &usize, request: &InputRequest, size: Rect) {
    let game = app.game.as_ref().expect("pending request implies a game");
    let title = format!("{} — decision", game.players[*player].name);
    let (title_line, body, error) = match request {
        InputRequest::ChooseCard { legal } => (
            String::from("Choose a card (1-9 to play, ↑/↓ + Enter, or the number)"),
            card_body(app, legal),
            app.error.clone(),
        ),
        InputRequest::ChooseChain { available } => (
            String::from("Which chain do you found? (↑/↓ to select, Enter to confirm)"),
            chain_body(app, available),
            app.error.clone(),
        ),
        InputRequest::FusionOrder { survivor, chains } => {
            let survivor_txt = survivor
                .map(|s| format!(" (survivor already fixed: {s})"))
                .unwrap_or_default();
            (
                format!("Order the fusion: press chain letters in fusion order{survivor_txt}"),
                fusion_order_body(app, chains),
                app.error.clone(),
            )
        }
        InputRequest::FusionStocks {
            dead,
            alive,
            max_exchange,
            max_sell,
            holder,
        } => (
            format!(
                "Your {dead} stocks: {} → {}  (< > exchange, ↑↓ sell, Enter)",
                holder, alive
            ),
            fusion_stock_body(app, *max_exchange, *max_sell),
            app.error.clone(),
        ),
        InputRequest::EndGame { description, .. } => (
            String::from("End the game now?  (y/n)"),
            vec![Line::from(description.as_str())],
            app.error.clone(),
        ),
        InputRequest::BuyStocks { table, .. } => (
            String::from("Buy stocks (≤3 total). 1-7 select row, < > adjust, Enter to confirm"),
            buy_body(app, table),
            app.error.clone(),
        ),
        InputRequest::Pass { can_redraw } => (
            String::from("No legal card. Redraw your hand? (y/n)"),
            vec![Line::from(if *can_redraw {
                "You may redraw a new card."
            } else {
                "You must pass."
            })],
            app.error.clone(),
        ),
    };

    let width = 70u16.min(size.width.saturating_sub(4));
    let height = (body.len() as u16 + 4).min(size.height.saturating_sub(2));
    let x = (size.width.saturating_sub(width)) / 2;
    let y = (size.height.saturating_sub(height)) / 2;
    let area = Rect::new(x, y, width, height);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Span::styled(
            title,
            Style::default().add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(Color::Black));
    let mut lines = vec![Line::from(title_line.as_str()), Line::from("")];
    lines.extend(body);
    if let Some(err) = &error {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("  {err}"),
            Style::default().fg(Color::Red),
        )));
    }
    lines.push(Line::from(""));
    let para = Paragraph::new(lines)
        .block(block)
        .style(Style::default().bg(Color::Black));
    frame.render_widget(para, area);
}

fn card_body<'a>(app: &'a App, legal: &'a [crate::core::board::Position]) -> Vec<Line<'a>> {
    let mut out = Vec::new();
    for (i, pos) in legal.iter().enumerate() {
        let marker = if i == app.selected_card { ">" } else { " " };
        let desc = app
            .game
            .as_ref()
            .and_then(|g| {
                g.players
                    .iter()
                    .find(|p| p.name == app.human_name)
                    .and_then(|p| p.analyzed_cards.iter().find(|c| c.position == *pos))
            })
            .map(|c| match &c.place_hotel_case {
                crate::core::rules::PlaceHotelCase::SingleHotel => "single hotel".to_string(),
                crate::core::rules::PlaceHotelCase::NewChain(p) => {
                    format!("found a {}-hotel chain", p.len())
                }
                crate::core::rules::PlaceHotelCase::ExtendsChain(chain, p) => {
                    format!("extend {chain} by {}", p.len())
                }
                crate::core::rules::PlaceHotelCase::Fusion(chains, _) => {
                    format!("FUSION of {} chains", chains.len())
                }
                crate::core::rules::PlaceHotelCase::Illegal(_) => "illegal".to_string(),
            })
            .unwrap_or_default();
        let style = if i == app.selected_card {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        out.push(Line::from(vec![
            Span::styled(format!(" {marker} {}. {pos} ", i + 1), style),
            Span::styled(desc, style),
        ]));
    }
    out
}

fn chain_body<'a>(app: &'a App, available: &'a [HotelChain]) -> Vec<Line<'a>> {
    available
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let marker = if i == app.selected_chain { ">" } else { " " };
            let style = if i == app.selected_chain {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            Line::from(vec![
                Span::styled(
                    format!("  {marker} {} ", c.identifier()),
                    style.fg(chain_color(c)),
                ),
                Span::styled(c.name(), style),
                Span::styled(
                    format!("  [{:?}]", c.price_level()),
                    style.fg(price_color(c.price_level())),
                ),
            ])
        })
        .collect()
}

fn fusion_order_body<'a>(app: &'a App, chains: &'a [HotelChain]) -> Vec<Line<'a>> {
    let mut out = Vec::new();
    let selected: &Vec<HotelChain> = &app.fusion_order_sel;
    out.push(Line::from(Span::styled(
        format!(
            "  chosen so far: {}",
            selected
                .iter()
                .map(|c| format!("{}", c.identifier()))
                .collect::<Vec<_>>()
                .join(" → ")
        ),
        Style::default().fg(Color::Cyan),
    )));
    for c in chains {
        let done = selected.contains(c);
        let style = if done {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(chain_color(c))
        };
        out.push(Line::from(vec![
            Span::styled(format!("  {} ", c.identifier()), style),
            Span::styled(c.name().to_string(), style),
            Span::styled(
                format!(
                    "  (len {})",
                    app.game
                        .as_ref()
                        .map(|g| g.chains.chain_length(c))
                        .unwrap_or(0)
                ),
                style,
            ),
        ]));
    }
    out
}

fn fusion_stock_body(app: &App, max_exchange: u32, max_sell: u32) -> Vec<Line<'_>> {
    vec![
        Line::from(format!(
            "  Exchange: {}  (even, 0..={max_exchange})   ← →  to change",
            app.fusion_exchange
        )),
        Line::from(format!(
            "  Sell:     {}   (0..={})   ↑ ↓  to change",
            app.fusion_sell,
            max_sell.saturating_sub(app.fusion_exchange)
        )),
        Line::from(format!(
            "  Keep:     {}",
            max_sell
                .saturating_sub(app.fusion_exchange)
                .saturating_sub(app.fusion_sell)
        )),
    ]
}

fn buy_body<'a>(app: &'a App, table: &'a [crate::core::game::BuyRow]) -> Vec<Line<'a>> {
    let mut out = Vec::new();
    for (i, row) in table.iter().enumerate() {
        let marker = if i == app.buy_row { ">" } else { " " };
        let count = app.buy_counts.get(i).copied().unwrap_or(0);
        let style = if i == app.buy_row {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        out.push(Line::from(vec![
            Span::styled(format!(" {marker} {}. {} ", i + 1, row.chain), style),
            Span::styled(
                format!(
                    "price {:>5} €   bank {:>2}   max {:>2}   → buy {}",
                    row.price, row.available, row.max_affordable, count
                ),
                style,
            ),
        ]));
    }
    out
}

// ---------------------------------------------------------------------------
// Game over
// ---------------------------------------------------------------------------

fn render_gameover(frame: &mut Frame, app: &App, size: Rect) {
    let result: FinalResult = app
        .final_result
        .clone()
        .expect("game over screen requires a final result");
    let mut lines = vec![Line::from(Span::styled(
        "Final ranking",
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::Cyan),
    ))];
    lines.push(Line::from(""));
    for (i, (name, money)) in result.ranking.iter().enumerate() {
        let (label, color) = match i {
            0 => (
                Span::styled(
                    "1. ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Color::Yellow,
            ),
            1 => (
                Span::styled(
                    "2. ",
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::BOLD),
                ),
                Color::Gray,
            ),
            2 => (
                Span::styled(
                    "3. ",
                    Style::default()
                        .fg(Color::Rgb(191, 137, 112))
                        .add_modifier(Modifier::BOLD),
                ),
                Color::Rgb(191, 137, 112),
            ),
            _ => (Span::raw(format!("{i}  ")), Color::DarkGray),
        };
        lines.push(Line::from(vec![
            label,
            Span::styled(format!("{name: <20}"), Style::default().fg(color)),
            Span::styled(format!("{money} €"), Style::default().fg(color)),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("Winner: {}", result.winner),
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::Yellow),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from("  q — quit"));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("Game Over");
    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Left),
        size.inner(&ratatui::layout::Margin::new(4, 2)),
    );
}

// ---------------------------------------------------------------------------
// Demo
// ---------------------------------------------------------------------------

fn render_demo(frame: &mut Frame, app: &App, size: Rect) {
    let demo = match &app.demo {
        Some(d) => d,
        None => return,
    };
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(10),
        ])
        .split(size);
    let title = Paragraph::new(" ").block(
        Block::default().title(Span::styled(
            "Acquire — Demo board (q to quit)",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
    );
    frame.render_widget(title, root[0]);

    // Board.
    render_demo_board(frame, demo, root[1]);
    // Table (reuse the play table via a temporary app is overkill; render a simple one).
    render_demo_table(frame, demo, root[2]);
}

fn render_demo_board(frame: &mut Frame, demo: &DemoState, area: Rect) {
    let no_highlight: Option<crate::core::board::Position> = None;
    let empty: Vec<crate::core::board::Position> = Vec::new();

    let mut rows: Vec<Row> = Vec::new();
    for (r, row) in demo.board.pieces.iter().enumerate() {
        let letter = crate::core::board::letter::LETTERS[r];
        let mut cells = vec![
            Cell::from(letter.to_string()).style(Style::default().add_modifier(Modifier::BOLD))
        ];
        for piece in row {
            cells.push(board_cell(piece, &no_highlight, &empty));
        }
        rows.push(Row::new(cells));
    }

    let mut widths: Vec<Constraint> = vec![Constraint::Length(2)];
    for _ in 0..12 {
        widths.push(Constraint::Length(3));
    }

    let table = Table::new(rows, widths).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Board"),
    );
    frame.render_widget(table, area);
}

fn render_demo_table(frame: &mut Frame, demo: &DemoState, area: Rect) {
    let chains = &demo.chains;
    let bank = &demo.bank;
    let mut rows = Vec::new();
    for chain in HotelChain::iterator() {
        let active = chains.chain_status(chain);
        let base = if active {
            Style::default()
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let price = if active {
            Bank::stock_price(chains, chain)
        } else {
            0
        };
        rows.push(Row::new(vec![
            Cell::from(chain.name()).style(if active {
                base.fg(chain_color(chain))
            } else {
                base
            }),
            Cell::from(chains.chain_length(chain).to_string()).style(base),
            Cell::from(chains.price_range(chain).trim().to_string()).style(base),
            Cell::from(if active {
                bank.stocks_available(chain, chains).to_string()
            } else {
                "-".to_string()
            })
            .style(base),
            Cell::from(format!("{price} €")).style(base),
        ]));
    }
    let header = Row::new(vec![
        Cell::from("Chain"),
        Cell::from("Len"),
        Cell::from("Range"),
        Cell::from("Bank"),
        Cell::from("Price"),
    ]);
    let table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Length(5),
            Constraint::Length(9),
            Constraint::Length(6),
            Constraint::Length(8),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title("Chains"));
    frame.render_widget(table, area);
}

// ---------------------------------------------------------------------------
// LAN placeholder
// ---------------------------------------------------------------------------

fn render_lan(frame: &mut Frame, app: &App, size: Rect) {
    let _ = app;
    let text = Paragraph::new(vec![
        Line::from(Span::styled(
            "Multiplayer is not implemented in this build.",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("The LAN wire-protocol specification lives in `src/network.rs` so a"),
        Line::from("future backend can be slotted in without touching the game logic."),
        Line::from(""),
        Line::from("  q — quit"),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("LAN / Multiplayer"),
    );
    frame.render_widget(text, size.inner(&ratatui::layout::Margin::new(4, 2)));
}

// ---------------------------------------------------------------------------
// Overlays
// ---------------------------------------------------------------------------

fn render_infocard(frame: &mut Frame, size: Rect) {
    // Port of the old print_info_card: stock value per chain level + bonuses.
    let levels: [&str; 9] = [
        "  2  ", "  3  ", "  4  ", "  5  ", " 6-10", "11-20", "21-30", "31-40", "41++ ",
    ];
    let base = [200u32, 300, 400, 500, 600, 700, 800, 900, 1000, 1100, 1200];
    let mut lines = vec![Line::from(Span::styled(
        "Stock value per chain length (€)",
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::Cyan),
    ))];
    lines.push(Line::from(
        "  L=Low  M=Medium  H=High     10× = largest  5× = second largest",
    ));
    for (i, price) in base.iter().enumerate() {
        let low = if i <= 8 { levels[i] } else { "  -  " };
        let medium = if (1..=9).contains(&i) {
            levels[i - 1]
        } else {
            "  -  "
        };
        let high = if (2..=10).contains(&i) {
            levels[i - 2]
        } else {
            "  -  "
        };
        lines.push(Line::from(vec![
            Span::styled(format!("  {low}"), Style::default().fg(Color::Red)),
            Span::styled(format!("  {medium}"), Style::default().fg(Color::Yellow)),
            Span::styled(format!("  {high}"), Style::default().fg(Color::Green)),
            Span::raw(format!("   {:>5}   ", price)),
            Span::styled(
                format!("  10× {:>6}", price * 10),
                Style::default().fg(Color::Gray),
            ),
            Span::styled(
                format!("  5× {:>6}", price * 5),
                Style::default().fg(Color::Gray),
            ),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from("  Esc/q — close"));

    let width = 78u16.min(size.width.saturating_sub(2));
    let height = (lines.len() as u16 + 2).min(size.height.saturating_sub(2));
    let x = (size.width.saturating_sub(width)) / 2;
    let y = (size.height.saturating_sub(height)) / 2;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("Stock info card");
    frame.render_widget(
        Paragraph::new(lines).block(block),
        Rect::new(x, y, width, height),
    );
}

fn render_help(frame: &mut Frame, size: Rect) {
    let lines = vec![
        Line::from(Span::styled(
            "Help",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Cyan),
        )),
        Line::from(""),
        Line::from("  Global:  i info card · h help · l toggle log · q quit"),
        Line::from("  Choose card:   1-9 play · ↑/↓ select · Enter play"),
        Line::from("  Choose chain:  ↑/↓ select · Enter confirm (or press the number)"),
        Line::from("  Fusion order:  press chain letters in fusion order (last = survivor)"),
        Line::from("  Fusion stocks: < > exchange (even) · ↑↓ sell · Enter confirm"),
        Line::from("  Buy stocks:    1-7 select row · < > adjust · Enter confirm"),
        Line::from("  End game:      y yes · n no"),
        Line::from(""),
        Line::from("  Rules: place a card each turn, then buy up to 3 stocks. Found 2+"),
        Line::from("  adjacent hotels to start a chain. Fuse when a card bridges 2+ chains."),
        Line::from("  Majority shareholders get 10×/5× bonuses. Highest money wins."),
        Line::from(""),
        Line::from("  Esc/q — close"),
    ];
    let width = 74u16.min(size.width.saturating_sub(2));
    let height = (lines.len() as u16 + 2).min(size.height.saturating_sub(2));
    let x = (size.width.saturating_sub(width)) / 2;
    let y = (size.height.saturating_sub(height)) / 2;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("Help");
    frame.render_widget(
        Paragraph::new(lines).block(block),
        Rect::new(x, y, width, height),
    );
}
