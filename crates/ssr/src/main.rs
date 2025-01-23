use std::{
    io::{stdout, Write},
    process::ExitCode,
};

use anyhow::{ensure, Result};
use clap::{Parser, Subcommand};
use ratatui::{
    prelude::{CrosstermBackend, *},
    widgets::*,
    Terminal,
};
use ratatui_inputs::ResultKind;
use s_text_input_f::BlocksWithAnswer;
use ssr_core::tasks_facade::TasksFacade;

type Task = ssr_algorithms::fsrs::Task;
type Facade<'a> = ssr_facade::Facade<'a, Task>;

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    action: Option<Action>,
}

#[derive(Subcommand, Debug)]
enum Action {
    Add { content: String },
}

const PATH: &str = "storage.json";

fn main() -> Result<ExitCode> {
    let args = Args::parse();

    const DEFAULT_DESIRED_RETENTION: f64 = 0.85;
    let mut storage = {
        if std::fs::exists(PATH)? {
            let file = Box::new(std::fs::read_to_string(PATH)?).leak();
            serde_json::from_str(file)?
        } else {
            Facade::new("test_name".into(), DEFAULT_DESIRED_RETENTION)
        }
    };

    let success = if let Some(action) = args.action {
        match action {
            Action::Add { content } => {
                let a = s_text_input_f_parser::parse_blocks(&content);
                match a {
                    Ok(blocks) => {
                        if blocks.answer.iter().map(|x| x.len()).sum::<usize>() == 0 {
                            println!("Task must contain interactive elements.");
                            false
                        } else {
                            let task = Task::new(blocks.blocks, blocks.answer);
                            storage.insert(task);
                            println!("Task added");
                            true
                        }
                    }
                    Err(errs) => {
                        for err in errs {
                            println!("Parsing error: {err}.");
                        }
                        false
                    }
                }
            }
        }
    } else {
        application(&mut storage)?;
        true
    };

    save(PATH, &storage)?;

    if success {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::FAILURE)
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Submenu {
    CompleteTask,
    CreateTask,
    ModifyDesiredRetention,
    Optimize,
    Save,
}

fn application(storage: &mut Facade) -> Result<()> {
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout())).unwrap();
    let alt = alternate_screen_wrapper::AlternateScreen::enter()?.bracketed_paste()?;

    loop {
        let submenu = {
            storage.find_tasks_to_recall();
            let request = vec![s_text_input_f::Block::OneOf(vec![
                format!("complete task ({})", {
                    let to_complete = storage.tasks_to_complete();
                    if to_complete > 0 {
                        to_complete.to_string()
                    } else {
                        let until = storage.until_next_repetition();
                        if let Some(until) = until {
                            format!("0; {:.2}h", until.as_secs_f64() / 3600.)
                        } else {
                            0.to_string()
                        }
                    }
                }),
                "create task".into(),
                format!(
                    "desired retention ({:.0}%)",
                    (storage.get_desired_retention() * 100.).floor()
                ),
                "optimize".into(),
                "save".into(),
            ])];
            let (result_kind, answer) = ratatui_inputs::get_input(request, &mut |text| {
                terminal
                    .draw(|f| f.render_widget(Paragraph::new(text), f.area()))
                    .map(|_| ())
            })
            .unwrap()?;

            if result_kind == ResultKind::Canceled {
                break;
            }
            let answer: usize = answer[0][0].parse()?;
            [
                Submenu::CompleteTask,
                Submenu::CreateTask,
                Submenu::ModifyDesiredRetention,
                Submenu::Optimize,
                Submenu::Save,
            ][answer]
        };
        match submenu {
            Submenu::CompleteTask => {
                complete_task(storage, &mut terminal);
            }
            Submenu::CreateTask => {
                if let Some(blocks_with_answer) = get_blocks_with_answer(&mut terminal)? {
                    storage.create_task(blocks_with_answer);
                }
            }
            Submenu::ModifyDesiredRetention => {
                if let Some(desired_retention) = get_desired_retention(&mut terminal)? {
                    storage.set_desired_retention(desired_retention);
                }
            }
            Submenu::Optimize => {
                terminal.draw(|f| {
                    f.render_widget(ratatui::widgets::Paragraph::new("Optimizing"), f.area());
                })?;
                if let Err(err) = storage.optimize() {
                    todo!()
                }
            }
            Submenu::Save => save(PATH, storage)?,
        }
    }
    drop(alt);
    Ok(())
}

fn get_desired_retention(terminal: &mut Terminal<impl Backend>) -> Result<Option<f64>> {
    fn parse_desired_retention(input: &str) -> Result<f64> {
        use std::str::FromStr;
        let number = f64::from_str(input)?;
        ensure!(number.is_finite(), "input must be \"normal\" number");
        ensure!(number > 0.0, "desired retention must be greater than 0%");
        ensure!(number < 1.0, "desired retention must be less than 100%");
        Ok(number)
    }

    let (result_kind, user_input) = ratatui_inputs::get_text_input(&mut |styled, raw| {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(1), Constraint::Fill(1)]);
        terminal
            .draw(|f| {
                let layout = layout.split(f.area());

                let input_block = ratatui::widgets::Block::bordered()
                    .border_type(ratatui::widgets::BorderType::Rounded);
                let input_area = input_block.inner(layout[0]);
                let support_block = ratatui::widgets::Block::new().padding(Padding::uniform(1));
                let support_area = support_block.inner(layout[1]);

                f.render_widget(input_block, layout[0]);
                f.render_widget(
                    ratatui::widgets::Paragraph::new(styled).wrap(Wrap { trim: true }),
                    input_area,
                );
                f.render_widget(support_block, layout[1]);

                let support_text = match parse_desired_retention(&raw) {
                    Ok(number) => format!("{:.2}%", number * 100.),
                    Err(err) => format!("Error: {err}."),
                };
                f.render_widget(ratatui::widgets::Paragraph::new(support_text), support_area);
            })
            .map(|_| ())
    })?;
    match result_kind {
        ResultKind::Ok => Ok(user_input.parse().ok()),
        ResultKind::Canceled => Ok(None),
        _ => unreachable!(),
    }
}

fn get_blocks_with_answer(
    terminal: &mut Terminal<impl Backend>,
) -> Result<Option<BlocksWithAnswer>> {
    Ok(ratatui_inputs::get_blocks(&mut |styled, support_text| {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(1), Constraint::Fill(1)]);
        terminal
            .draw(|f| {
                let layout = layout.split(f.area());

                let input_block = ratatui::widgets::Block::bordered()
                    .border_type(ratatui::widgets::BorderType::Rounded);
                let input_area = input_block.inner(layout[0]);

                let support_block = ratatui::widgets::Block::new().padding(Padding::uniform(1));
                let support_area = support_block.inner(layout[1]);

                f.render_widget(input_block, layout[0]);
                f.render_widget(
                    ratatui::widgets::Paragraph::new(styled).wrap(Wrap { trim: true }),
                    input_area,
                );
                f.render_widget(support_block, layout[1]);
                f.render_widget(ratatui::widgets::Paragraph::new(support_text), support_area);
            })
            .map(|_| ())
    })?)
}

fn complete_task(storage: &mut Facade, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) {
    let _ = storage.complete_task(&mut |id, blocks| {
        let (_result_kind, answer) = ratatui_inputs::get_input(blocks, &mut |mut text| {
            use ratatui::style::Stylize;
            text.push_line("");
            text.push_line(format!("ID {id}").dark_gray().italic());

            terminal
                .draw(|f| f.render_widget(text, f.area()))
                .map(|_| ())
        })
        .transpose()?
        .unwrap_or((ResultKind::Ok, vec![vec![]]));
        Ok(answer)
    });
}

// FIXME: first create file, than rename it to `path` to not corrupt data
fn save(path: &str, storage: &Facade) -> Result<()> {
    writeln!(
        std::fs::File::create(path)?,
        "{}",
        serde_json::to_string_pretty(storage)?
    )?;
    Ok(())
}
