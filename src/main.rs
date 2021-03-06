extern crate alectro;
extern crate futures;
extern crate irc;
extern crate termion;
extern crate tokio_core;

use std::env;

use alectro::controller::{InputController, IrcController};
use alectro::input::AsyncKeyInput;
use alectro::view::UI;
use irc::client::prelude::*;
use tokio_core::reactor::Core;

fn main() {
    let ui = UI::new().unwrap();
    let mut reactor = Core::new().unwrap();

    let default_cfg = Config {
        nickname: Some(format!("alectro")),
        server: Some(format!("irc.pdgn.co")),
        use_ssl: Some(true),
        .. Default::default()
    };

    let cfg = match env::home_dir() {
        Some(mut path) => {
            path.push(".alectro");
            path.set_extension("toml");
            Config::load(path).unwrap_or(default_cfg)
        },
        None => default_cfg,
    };

    for chan in &cfg.channels() {
        ui.new_chat_buf(chan).unwrap();
    }

    let irc_server = IrcServer::from_config(cfg).unwrap();
    irc_server.identify().unwrap();

    let irc_controller = IrcController::new(ui.clone());
    let output_future = irc_server.stream().map_err(|e| e.into()).for_each(|message| {
        irc_controller.handle_message(message)?;
        irc_controller.ui().draw_all()
    });

    let input_controller = InputController::new(irc_server, ui);
    let input_rx = AsyncKeyInput::new();
    let input_future = input_rx.for_each(|event| {
        input_controller.handle_event(event)?;
        input_controller.ui().draw_all()
    });

    reactor.run(output_future.join(input_future)).unwrap();
}
