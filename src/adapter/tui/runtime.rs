use crate::adapter::tui::app::{Screen, TuiApp};
use crate::adapter::tui::ui::draw;
use crate::application::ApplicationState;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use std::io::{self, Stdout};
use std::time::Duration;

type Backend = ratatui::backend::CrosstermBackend<Stdout>;
type TuiTerminal = Terminal<Backend>;

pub async fn run() -> anyhow::Result<()> {
    let state = ApplicationState::new().await?;
    let mut app = TuiApp::new(state.quote_port.clone());
    if let Err(err) = app.reload_full().await {
        app.status = format!("load failed: {err}");
    }

    let mut terminal = setup_terminal()?;
    let loop_result = run_loop(&mut terminal, &mut app).await;
    let restore_result = restore_terminal(&mut terminal);
    loop_result?;
    restore_result?;
    Ok(())
}

fn setup_terminal() -> anyhow::Result<TuiTerminal> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut TuiTerminal) -> anyhow::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

async fn run_loop(terminal: &mut TuiTerminal, app: &mut TuiApp) -> anyhow::Result<()> {
    loop {
        terminal.draw(|frame| draw(frame, app))?;
        if app.should_quit {
            break;
        }

        if !event::poll(Duration::from_millis(200))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        if app.command_mode {
            // 命令模式下只处理命令行编辑与执行，不触发普通导航按键。
            match key.code {
                KeyCode::Enter => app.execute_command(),
                KeyCode::Esc => app.cancel_command_mode(),
                KeyCode::Backspace => app.pop_command_char(),
                KeyCode::Char(ch) => app.append_command_char(ch),
                _ => {}
            }
            continue;
        }

        if app.screen == Screen::Detail {
            match key.code {
                KeyCode::Char('q') => app.should_quit = true,
                KeyCode::Char(':') => app.enter_command_mode(),
                KeyCode::Down | KeyCode::Char('j') => app.scroll_detail_down(),
                KeyCode::Up | KeyCode::Char('k') => app.scroll_detail_up(),
                _ => {}
            }
            continue;
        }

        match key.code {
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Char(':') => app.enter_command_mode(),
            KeyCode::Char('r') => {
                if let Err(err) = app.reload_full().await {
                    app.status = format!("reload failed: {err}");
                } else {
                    app.reset_status();
                }
            }
            KeyCode::Down | KeyCode::Char('j') => app.select_next(),
            KeyCode::Up | KeyCode::Char('k') => app.select_prev(),
            KeyCode::Enter => app.open_detail(),
            KeyCode::Char('J') => app.select_last(),
            KeyCode::Char('K') => app.select_first(),
            KeyCode::Right | KeyCode::Char('l') => {
                if app.page < app.max_page() {
                    app.page += 1;
                    if let Err(err) = app.reload_page().await {
                        if app.page > 1 {
                            app.page -= 1;
                        }
                        app.status = format!("next page failed: {err}");
                    } else {
                        app.reset_status();
                    }
                }
            }
            KeyCode::Char('L') => {
                let last = app.max_page();
                if app.page < last {
                    app.page = last;
                    if let Err(err) = app.reload_page().await {
                        app.status = format!("last page failed: {err}");
                    } else {
                        app.reset_status();
                    }
                }
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if app.page > 1 {
                    app.page -= 1;
                    if let Err(err) = app.reload_page().await {
                        app.status = format!("prev page failed: {err}");
                    } else {
                        app.reset_status();
                    }
                }
            }
            KeyCode::Char('H') => {
                if app.page > 1 {
                    app.page = 1;
                    if let Err(err) = app.reload_page().await {
                        app.status = format!("first page failed: {err}");
                    } else {
                        app.reset_status();
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}
