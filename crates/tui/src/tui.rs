use std::{
    ops::{Deref, DerefMut},
    panic::{set_hook, take_hook},
};

use crossterm::{
    cursor,
    event::{Event as CrosstermEvent, KeyEvent, KeyEventKind, MouseEvent},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::{FutureExt, StreamExt};
use lib::Error;
use ratatui::backend::CrosstermBackend as Backend;
use tokio::{
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Debug)]
pub enum Event {
    Init,
    #[allow(dead_code)]
    Quit,
    Error,
    Tick,
    Render,
    FocusGained,
    FocusLost,
    Paste,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}

pub struct Tui {
    pub terminal: ratatui::Terminal<Backend<std::io::Stderr>>,
    pub task: JoinHandle<()>,
    pub cancellation_token: CancellationToken,
    pub event_rx: UnboundedReceiver<Event>,
    pub event_tx: UnboundedSender<Event>,
    pub frame_rate: f64,
    pub tick_rate: f64,
}

impl Tui {
    pub fn new() -> Result<Self, Error> {
        let tick_rate = 4.0;
        let frame_rate = 30.0;
        let terminal = ratatui::Terminal::new(Backend::new(std::io::stderr()))?;
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let cancellation_token = CancellationToken::new();
        let task = tokio::spawn(async {});
        Ok(Self {
            terminal,
            task,
            cancellation_token,
            event_rx,
            event_tx,
            frame_rate,
            tick_rate,
        })
    }

    pub fn start(&mut self) {
        let tick_delay = std::time::Duration::from_secs_f64(1.0 / self.tick_rate);
        let render_delay = std::time::Duration::from_secs_f64(1.0 / self.frame_rate);
        self.cancel();
        self.cancellation_token = CancellationToken::new();
        let _cancellation_token = self.cancellation_token.clone();
        let event_tx = self.event_tx.clone();
        self.task = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut tick_interval = tokio::time::interval(tick_delay);
            let mut render_interval = tokio::time::interval(render_delay);
            event_tx.send(Event::Init).unwrap();
            loop {
                let tick_delay = tick_interval.tick();
                let render_delay = render_interval.tick();
                let crossterm_event = reader.next().fuse();
                tokio::select! {
                  _ = _cancellation_token.cancelled() => {
                    break;
                  }
                  maybe_event = crossterm_event => {

                    match maybe_event {
                      Some(Ok(evt)) => {
                        match evt {
                          CrosstermEvent::Key(key) => {
                                // On Windows, when you press a key, 2 events are emitted:
                                //   - one with `KeyEventKind::Press`
                                //   - one with `KeyEventKind::Release`
                                // We only care of the `Press` kind.
                                if key.kind == KeyEventKind::Press {
                                    event_tx.send(Event::Key(key)).unwrap();
                                }
                          },
                          CrosstermEvent::Mouse(mouse) => {
                            event_tx.send(Event::Mouse(mouse)).unwrap();
                          },
                          CrosstermEvent::Resize(x, y) => {
                            event_tx.send(Event::Resize(x, y)).unwrap();
                          },
                          CrosstermEvent::FocusLost => {
                            event_tx.send(Event::FocusLost).unwrap();
                          },
                          CrosstermEvent::FocusGained => {
                            event_tx.send(Event::FocusGained).unwrap();
                          },
                          CrosstermEvent::Paste(_) => {
                            event_tx.send(Event::Paste).unwrap();
                          }
                        }
                      }
                      Some(Err(_)) => {
                        event_tx.send(Event::Error).unwrap();
                      }
                      None => {},
                    }
                  },
                  _ = tick_delay => {
                      event_tx.send(Event::Tick).unwrap();
                  },
                  _ = render_delay => {
                      event_tx.send(Event::Render).unwrap();
                  },
                }
            }
        });
    }

    pub fn stop(&self) -> Result<(), Error> {
        self.cancel();
        Ok(())
    }

    pub fn init_panic_hook(&self) {
        let original_hook = take_hook();
        set_hook(Box::new(move |panic_info| {
            Self::restore_tui().unwrap();
            original_hook(panic_info);
        }));
    }

    pub fn enter(&mut self) -> Result<(), Error> {
        self.init_panic_hook();
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(
            std::io::stderr(),
            EnterAlternateScreen,
            //EnableMouseCapture,
            cursor::Hide,
            //PushKeyboardEnhancementFlags(
            //    KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
            //        | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
            //)
        )?;
        self.start();
        Ok(())
    }

    pub fn exit(&mut self) -> Result<(), Error> {
        self.stop()?;
        if crossterm::terminal::is_raw_mode_enabled()? {
            self.flush()?;
            crossterm::execute!(
                std::io::stderr(),
                LeaveAlternateScreen,
                // DisableMouseCapture,
                cursor::Show,
                //PopKeyboardEnhancementFlags
            )?;
            crossterm::terminal::disable_raw_mode()?;
        }
        Ok(())
    }

    pub fn restore_tui() -> Result<(), Error> {
        if crossterm::terminal::is_raw_mode_enabled()? {
            crossterm::execute!(
                std::io::stderr(),
                LeaveAlternateScreen,
                //DisableMouseCapture,
                cursor::Show
            )?;
            crossterm::terminal::disable_raw_mode()?;
        }
        Ok(())
    }

    pub fn cancel(&self) {
        self.cancellation_token.cancel();
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.event_rx.recv().await
    }
}

impl Deref for Tui {
    type Target = ratatui::Terminal<Backend<std::io::Stderr>>;

    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

impl DerefMut for Tui {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        self.exit().unwrap();
    }
}
