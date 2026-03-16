use crate::adapter::tui::program::Program;
use crate::application::ApplicationState;

pub async fn run() -> anyhow::Result<()> {
    let application = ApplicationState::new().await?;
    let mut program = Program::new(application.quote_port.clone())?;
    program.load_initial();

    let loop_result = run_loop(&mut program);
    let restore_result = program.restore();
    loop_result?;
    restore_result?;
    Ok(())
}

fn run_loop(program: &mut Program) -> anyhow::Result<()> {
    while !program.quit {
        if program.redraw {
            program.draw()?;
        }
        program.tick()?;
    }
    Ok(())
}
