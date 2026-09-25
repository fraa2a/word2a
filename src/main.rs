pub mod ui;

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;
use ui::{GameStatus, Guess, LetterState, MAX_ATTEMPTS, UiState, WORD_LEN};

struct Game {
    word: String,
    running: bool,
    ui: UiState,
    dict: Vec<&'static str>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            word: String::new(),
            running: true,
            ui: UiState::default(),
            dict: vec![],
        }
    }

    fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        self.start_game();
        while self.running {
            terminal.draw(|frame| ui::render(frame, &self.ui))?;

            if let Event::Key(key) = event::read()?
                && key.kind != KeyEventKind::Release
            {
                self.handle_key(key.code);
            }
        }
        Ok(())
    }

    fn load_dict(&mut self) {
        let dictionary = include_str!("../dict/dict.txt");
        let words: Vec<&str> = dictionary.lines().collect();

        self.dict = words
    }

    fn fetch_word(&self) -> String {
        let words = &self.dict;
        words[rand::random_range(0..words.len())].to_string()
    }

    fn start_game(&mut self) {
        self.load_dict();
        let mut next_word = self.fetch_word();
        while next_word == self.word {
            next_word = self.fetch_word();
        }
        self.word = next_word;
        self.ui = UiState::default();
    }

    fn handle_key(&mut self, key: KeyCode) {
        if key == KeyCode::Esc {
            self.running = false;
            return;
        }
        if self.ui.status != GameStatus::Playing {
            if key == KeyCode::Enter {
                self.start_game()
            };
            return;
        }

        match key {
            KeyCode::Char(letter)
                if letter.is_ascii_alphabetic() && self.ui.current_input.len() < WORD_LEN =>
            {
                self.ui.current_input.push(letter.to_ascii_lowercase());
                self.ui.message = None;
            }
            KeyCode::Backspace => {
                self.ui.current_input.pop();
                self.ui.message = None;
            }
            KeyCode::Enter => self.submit_guess(),
            _ => {}
        }
    }

    fn submit_guess(&mut self) {
        if self.ui.current_input.len() != WORD_LEN {
            self.ui.message = Some(format!("Scrivi una parola di {WORD_LEN} lettere"));
            return;
        }

        let guess = std::mem::take(&mut self.ui.current_input);

        if !(self.dict.contains(&guess.as_str())) {
            self.ui.message = Some(format!("Parola non riconosciuta"));
            return;
        }

        let result = score_guess(&guess, &self.word);
        let won = result.iter().all(|state| *state == LetterState::Correct);
        self.ui.guesses.push(Guess {
            word: guess,
            result,
        });
        self.ui.status = if won {
            GameStatus::Won
        } else if self.ui.guesses.len() >= MAX_ATTEMPTS {
            GameStatus::Lost
        } else {
            GameStatus::Playing
        };
        self.ui.message = None;
    }
}

fn score_guess(guess: &str, answer: &str) -> [LetterState; WORD_LEN] {
    let guess = guess.as_bytes();
    let answer = answer.as_bytes();
    let mut result = [LetterState::Absent; WORD_LEN];
    let mut used = [false; WORD_LEN];

    for index in 0..WORD_LEN {
        if guess[index] == answer[index] {
            result[index] = LetterState::Correct;
            used[index] = true;
        }
    }
    for index in 0..WORD_LEN {
        if result[index] == LetterState::Correct {
            continue;
        }
        if let Some(position) =
            (0..WORD_LEN).find(|&position| !used[position] && guess[index] == answer[position])
        {
            result[index] = LetterState::Present;
            used[position] = true;
        }
    }
    result
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let game = Game::new();

    let result = game.run(&mut terminal);

    ratatui::restore();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_letters_use_each_answer_letter_once() {
        assert_eq!(
            score_guess("baaac", "aabbb"),
            [
                LetterState::Present,
                LetterState::Correct,
                LetterState::Present,
                LetterState::Absent,
                LetterState::Absent,
            ]
        );
    }

    #[test]
    fn enter_starts_a_clean_round_after_game_over() {
        let mut game = Game::new();
        game.word = "acqua".to_string();
        game.ui.status = GameStatus::Won;
        game.ui.current_input = "prova".to_string();
        game.ui.message = Some("vecchio messaggio".to_string());

        game.handle_key(KeyCode::Enter);

        assert_eq!(game.ui.status, GameStatus::Playing);
        assert!(game.ui.current_input.is_empty());
        assert!(game.ui.guesses.is_empty());
        assert!(game.ui.message.is_none());
        assert_eq!(game.word.len(), WORD_LEN);
        assert_ne!(game.word, "acqua");
    }
}
