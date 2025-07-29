use crossterm::event::{Event as CrosstermEvent, EventStream, KeyCode, KeyEventKind, KeyModifiers};
use futures::{FutureExt, StreamExt};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Move {
    First,
    Last,
    Next,
    Previous,
    JumpUp,
    JumpDown,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Events {
    Quit,
    Back,
    Search,
    Confirm,
    Filter,
    Select,
    SelectUpgradables,
    Navigate(Move),
    Tab(Move),
    Sync,
}

#[derive(Debug)]
pub struct KeyboardEvent {
    pub event: Option<Events>,
    pub raw: CrosstermEvent,
}

impl Default for KeyboardEvent {
    fn default() -> Self {
        Self {
            event: None,
            raw: CrosstermEvent::FocusGained,
        }
    }
}

pub async fn read_event() -> KeyboardEvent {
    let mut reader = EventStream::new();

    loop {
        let event = reader.next().fuse();

        if let Some(Ok(raw)) = event.await {
            if let CrosstermEvent::Key(key) = raw {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                let event: Option<Events> = match (key.modifiers, key.code) {
                    (_, KeyCode::Char('j')) | (_, KeyCode::Down) => {
                        Some(Events::Navigate(Move::Next))
                    }
                    (_, KeyCode::Char('k')) | (_, KeyCode::Up) => {
                        Some(Events::Navigate(Move::Previous))
                    }
                    (_, KeyCode::Char('g')) | (_, KeyCode::Home) => {
                        Some(Events::Navigate(Move::First))
                    }
                    (_, KeyCode::Char('G')) | (_, KeyCode::End) => {
                        Some(Events::Navigate(Move::Last))
                    }
                    (_, KeyCode::Tab) => Some(Events::Tab(Move::Next)),
                    (_, KeyCode::BackTab) => Some(Events::Tab(Move::Previous)),
                    (_, KeyCode::Char('x')) => Some(Events::Select),
                    (_, KeyCode::Char('/')) => Some(Events::Search),
                    (_, KeyCode::Char('q')) => Some(Events::Quit),
                    (_, KeyCode::Esc) => Some(Events::Back),
                    (_, KeyCode::Enter) => Some(Events::Confirm),
                    (KeyModifiers::CONTROL, KeyCode::Char('u')) => {
                        Some(Events::Navigate(Move::JumpUp))
                    }
                    (KeyModifiers::CONTROL, KeyCode::Char('d')) => {
                        Some(Events::Navigate(Move::JumpDown))
                    }
                    (KeyModifiers::ALT, KeyCode::Char('u')) => Some(Events::Filter),
                    (KeyModifiers::SHIFT, KeyCode::Char('X')) => Some(Events::SelectUpgradables),
                    (KeyModifiers::SHIFT, KeyCode::Char('S')) => Some(Events::Sync),
                    _ => None,
                };

                return KeyboardEvent { event, raw };
            }
        }
    }
}
