//La logica può aggiornare `UiState` e richiamare `render`.
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
};

pub const WORD_LEN: usize = 5;
pub const MAX_ATTEMPTS: usize = 6;

const EMPTY: Color = Color::Rgb(48, 58, 75);
const TEXT: Color = Color::Rgb(240, 240, 232);
const MUTED: Color = Color::Rgb(140, 153, 171);
const ACCENT: Color = Color::Rgb(245, 193, 104);
const CORRECT: Color = Color::Rgb(87, 169, 131);
const PRESENT: Color = Color::Rgb(218, 171, 92);
const ABSENT: Color = Color::Rgb(111, 120, 135);
const TILE_INK: Color = Color::Rgb(20, 26, 36);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LetterState {
    Absent,
    Present,
    Correct,
}

#[derive(Clone, Debug)]
pub struct Guess {
    pub word: String,
    pub result: [LetterState; WORD_LEN],
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GameStatus {
    #[default]
    Playing,
    Won,
    Lost,
}

#[derive(Clone, Debug, Default)]
pub struct UiState {
    // tentativi gia' fatti
    pub guesses: Vec<Guess>,
    pub current_input: String,
    // se message assente mette il messaggio legato al GameStatus
    pub message: Option<String>,
    pub status: GameStatus,
}

pub fn render(frame: &mut Frame, state: &UiState) {
    let area = frame.area();

    if area.width < 34 || area.height < 14 {
        let notice = Paragraph::new("✦ Allarga il terminale per giocare ✦")
            .style(Style::default().fg(ACCENT))
            .alignment(Alignment::Center);
        frame.render_widget(
            notice,
            Rect::new(area.x, area.y + area.height / 2, area.width, 1),
        );
        return;
    }

    let tall = area.height >= 37;
    let compact = area.height < 23;
    let tile_height = if tall {
        3
    } else if compact {
        1
    } else {
        2
    };
    let row_gap = u16::from(tall);
    let board_height = MAX_ATTEMPTS as u16 * tile_height + (MAX_ATTEMPTS as u16 - 1) * row_gap;
    let header_height = if compact { 2 } else { 3 };
    let keyboard_height = if compact { 3 } else { 5 };
    let base_height = header_height + 1 + board_height + 1 + keyboard_height + 1;
    let keyboard_gap = area
        .height
        .saturating_sub(base_height)
        .min(if tall { 3 } else { 2 });
    let content_height = base_height + keyboard_gap;
    let origin_y = area.y + area.height.saturating_sub(content_height) / 2;

    draw_header(frame, area, origin_y, compact);
    let board_y = origin_y + header_height + 1;
    draw_board(frame, area, board_y, tile_height, row_gap, state);

    let message_y = board_y + board_height;
    draw_message(frame, area, message_y, state);
    let keyboard_y = message_y + 1 + keyboard_gap;
    draw_keyboard(frame, area, keyboard_y, compact, state);

    let footer_y = keyboard_y + keyboard_height;
    let footer = if state.status != GameStatus::Playing {
        Line::from(vec![
            Span::styled("INVIO", Style::default().fg(ACCENT)),
            Span::styled(" nuova partita  ·  ", Style::default().fg(MUTED)),
            Span::styled("ESC", Style::default().fg(ACCENT)),
            Span::styled(" esci", Style::default().fg(MUTED)),
        ])
    } else if area.width < 45 {
        Line::from(vec![
            Span::styled("INVIO", Style::default().fg(ACCENT)),
            Span::styled(" invia · ", Style::default().fg(MUTED)),
            Span::styled("⌫", Style::default().fg(ACCENT)),
            Span::styled(" canc. · ", Style::default().fg(MUTED)),
            Span::styled("ESC", Style::default().fg(ACCENT)),
            Span::styled(" esci", Style::default().fg(MUTED)),
        ])
    } else {
        Line::from(vec![
            Span::styled(
                "INVIO",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" invia   ", Style::default().fg(MUTED)),
            Span::styled(
                "⌫",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" cancella   ", Style::default().fg(MUTED)),
            Span::styled(
                "ESC",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" esci", Style::default().fg(MUTED)),
        ])
    };
    let footer = Paragraph::new(footer).alignment(Alignment::Center);
    frame.render_widget(footer, Rect::new(area.x, footer_y, area.width, 1));
}

fn draw_header(frame: &mut Frame, area: Rect, y: u16, compact: bool) {
    let title = Paragraph::new(Line::from(vec![
        Span::styled("✦  ", Style::default().fg(ACCENT)),
        Span::styled(
            "WORD",
            Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "2A",
            Style::default().fg(CORRECT).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ✦", Style::default().fg(ACCENT)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(title, Rect::new(area.x, y, area.width, 1));
}

fn draw_board(
    frame: &mut Frame,
    area: Rect,
    y: u16,
    tile_height: u16,
    row_gap: u16,
    state: &UiState,
) {
    let tile_width = 5;
    let board_width = WORD_LEN as u16 * tile_width + (WORD_LEN as u16 - 1);
    let board_x = area.x + (area.width - board_width) / 2;

    for row in 0..MAX_ATTEMPTS {
        let row_y = y + row as u16 * (tile_height + row_gap);
        let guess = state.guesses.get(row);
        let current =
            guess.is_none() && row == state.guesses.len() && state.status == GameStatus::Playing;
        let letters: Vec<char> = if let Some(guess) = guess {
            guess.word.chars().collect()
        } else if current {
            state.current_input.chars().collect()
        } else {
            Vec::new()
        };

        for col in 0..WORD_LEN {
            let letter = letters
                .get(col)
                .copied()
                .unwrap_or(' ')
                .to_uppercase()
                .to_string();
            let result = guess.map(|guess| guess.result[col]);
            let filled = letters.get(col).is_some();
            let background = result.map(|state| match state {
                LetterState::Correct => CORRECT,
                LetterState::Present => PRESENT,
                LetterState::Absent => ABSENT,
            });
            let fg = if background.is_some() {
                TILE_INK
            } else if filled {
                TEXT
            } else {
                MUTED
            };
            let cell = Rect::new(
                board_x + col as u16 * (tile_width + 1),
                row_y,
                tile_width,
                tile_height,
            );
            let underline = if current && !filled { ACCENT } else { EMPTY };
            draw_tile(frame, cell, &letter, fg, underline, background);
        }
    }
}

fn draw_tile(
    frame: &mut Frame,
    cell: Rect,
    letter: &str,
    fg: Color,
    underline: Color,
    background: Option<Color>,
) {
    let display = if letter == " " { "·" } else { letter };
    let mut style = Style::default().fg(fg).add_modifier(Modifier::BOLD);
    if let Some(color) = background {
        style = style.bg(color);
        frame.render_widget(Block::default().style(Style::default().bg(color)), cell);
    }
    let center_x = cell.x + cell.width / 2;
    if cell.height == 1 {
        if background.is_none() {
            frame
                .buffer_mut()
                .set_string(center_x - 1, cell.y, "[", style);
            frame
                .buffer_mut()
                .set_string(center_x + 1, cell.y, "]", style);
        }
        frame
            .buffer_mut()
            .set_string(center_x, cell.y, display, style);
    } else {
        let text_y = cell.y + (cell.height - 1) / 2;
        frame
            .buffer_mut()
            .set_string(center_x, text_y, display, style);
        if background.is_none() {
            frame.render_widget(
                Paragraph::new("━━━━━")
                    .style(Style::default().fg(underline))
                    .alignment(Alignment::Center),
                Rect::new(cell.x, cell.y + cell.height - 1, cell.width, 1),
            );
        }
    }
}

fn draw_message(frame: &mut Frame, area: Rect, y: u16, state: &UiState) {
    let (message, color) = match (&state.message, state.status) {
        (Some(message), _) => (message.clone(), ACCENT),
        (None, GameStatus::Won) => ("Hai trovato la parola corretta ✦".to_string(), CORRECT),
        (None, GameStatus::Lost) => ("La prossima andrà meglio".to_string(), PRESENT),
        (None, GameStatus::Playing) => (
            format!(
                "Tentativo {:02} / {MAX_ATTEMPTS:02}",
                state.guesses.len() + 1
            ),
            MUTED,
        ),
    };
    frame.render_widget(
        Paragraph::new(message)
            .style(Style::default().fg(color))
            .alignment(Alignment::Center),
        Rect::new(area.x, y, area.width, 1),
    );
}

fn draw_keyboard(frame: &mut Frame, area: Rect, y: u16, compact: bool, state: &UiState) {
    let rows = ["QWERTYUIOP", "ASDFGHJKL", "ZXCVBNM"];
    let key_gap = u16::from(area.width >= 40);
    for (index, keys) in rows.iter().enumerate() {
        let row_y = y + index as u16 * if compact { 1 } else { 2 };
        let row_width = keys.chars().count() as u16 * (3 + key_gap) - key_gap;
        let row_x = area.x + (area.width - row_width) / 2;
        for (column, key) in keys.chars().enumerate() {
            let background = key_state(key, state).map(|result| match result {
                LetterState::Correct => CORRECT,
                LetterState::Present => PRESENT,
                LetterState::Absent => ABSENT,
            });
            let key_rect = Rect::new(row_x + column as u16 * (3 + key_gap), row_y, 3, 1);
            let mut style = Style::default().fg(TEXT).add_modifier(Modifier::BOLD);
            if let Some(color) = background {
                frame.render_widget(Block::default().style(Style::default().bg(color)), key_rect);
                style = style.fg(TILE_INK).bg(color);
            }
            frame
                .buffer_mut()
                .set_string(key_rect.x + 1, row_y, key.to_string(), style);
        }
    }
}

fn key_state(key: char, state: &UiState) -> Option<LetterState> {
    state
        .guesses
        .iter()
        .flat_map(|guess| guess.word.chars().zip(guess.result))
        .filter(|(letter, _)| letter.to_ascii_uppercase() == key)
        .map(|(_, result)| result)
        .max_by_key(|result| match result {
            LetterState::Absent => 0,
            LetterState::Present => 1,
            LetterState::Correct => 2,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn confirmed_tile_fills_its_area_without_coloring_the_screen() {
        let mut terminal = Terminal::new(TestBackend::new(40, 24)).unwrap();
        let state = UiState {
            guesses: vec![Guess {
                word: "abaco".to_string(),
                result: [LetterState::Correct; WORD_LEN],
            }],
            ..UiState::default()
        };

        terminal.draw(|frame| render(frame, &state)).unwrap();
        let buffer = terminal.backend().buffer();
        for y in 4..6 {
            for x in 5..10 {
                assert_eq!(buffer[(x, y)].bg, CORRECT);
            }
        }
        assert_eq!(buffer[(7, 4)].symbol(), "A");
        assert_eq!(buffer[(6, 4)].symbol(), " ");
        for x in 2..5 {
            assert_eq!(buffer[(x, 20)].bg, CORRECT);
        }
        assert_eq!(buffer[(3, 20)].symbol(), "A");
        assert_eq!(buffer[(2, 20)].symbol(), " ");
        assert_eq!(buffer[(0, 0)].bg, Color::Reset);
    }
}
