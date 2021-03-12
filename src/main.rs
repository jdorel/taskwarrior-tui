#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod app;
mod calendar;
mod config;
mod context;
mod help;
mod keyconfig;
mod table;
mod task_report;
mod util;
mod history;

use crate::util::{destruct_terminal, setup_terminal, Event, EventConfig, Events};
use clap::{App, Arg};
use std::env;
use std::error::Error;
use std::io::Write;
use std::panic;
use std::time::Duration;

use crate::util::Key;
use app::{AppMode, TTApp};

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_NAME: &str = env!("CARGO_PKG_NAME");

fn main() -> Result<(), Box<dyn Error>> {
    better_panic::install();
    let matches = App::new(APP_NAME)
        .version(APP_VERSION)
        .author("Dheepak Krishnamurthy <@kdheepak>")
        .about("A taskwarrior terminal user interface")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true),
        )
        .get_matches();

    let config = matches.value_of("config").unwrap_or("~/.taskrc");
    let r = tui_main(config);
    match r {
        Ok(_) => std::process::exit(0),
        Err(error) => {
            if error.to_string().to_lowercase().contains("no such file or directory") {
                eprintln!(
                    "[taskwarrior-tui error]: Unable to find executable `task`: {}. Check that taskwarrior is installed correctly and try again.", error
                );
            } else {
                eprintln!(
                    "[taskwarrior-tui error]: {}. Please report as a github issue on https://github.com/kdheepak/taskwarrior-tui",
                    error
                );
            }
            std::process::exit(1);
        }
    }
}

fn tui_main(_config: &str) -> Result<(), Box<dyn Error>> {
    // Terminal initialization
    let mut terminal = setup_terminal();

    panic::set_hook(Box::new(|panic_info| {
        destruct_terminal();
        better_panic::Settings::auto().create_panic_handler()(panic_info);
    }));

    // Setup event handlers
    let events = Events::with_config(EventConfig {
        tick_rate: Duration::from_millis(250),
    });

    let maybeapp = TTApp::new();
    match maybeapp {
        Ok(mut app) => {
            app.task_report_next();
            app.context_next();

            loop {
                terminal.draw(|mut frame| app.draw(&mut frame)).unwrap();

                // Handle input
                match events.next()? {
                    Event::Input(input) => {
                        let r = app.handle_input(input, &mut terminal, &events);
                        if r.is_err() {
                            destruct_terminal();
                            return r;
                        }
                    }
                    Event::Tick => {
                        let r = app.update(false);
                        if r.is_err() {
                            destruct_terminal();
                            return r;
                        }
                    }
                }

                if app.should_quit {
                    destruct_terminal();
                    break;
                }
            }

            Ok(())
        }
        Err(e) => {
            destruct_terminal();
            Err(e)
        }
    }
}
