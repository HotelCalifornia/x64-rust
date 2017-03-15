extern crate discord;
use discord::{Discord, State};
use discord::model::{Event, Message};
use std::env;
static PREFIX: &'static str = "*69";

struct Cmd {
    id: &'static str,
    cb: Box<Fn(Discord, &Message, Vec<&str>) -> Discord>,
}

fn main() {
    let mut discord = Discord::from_bot_token(&env::var("DISCORD_TOKEN").expect("Could not find token."))
        .expect("Login failed.");
    let commands: Vec<Cmd> = vec![
        Cmd {
            id: "_",
            cb: Box::new(|ctx: Discord, msg: &Message, args: Vec<&str>| -> Discord {
                let snd = format!("Unknown command `{}`\nUsage: `{} <cmd> <args>`\nFor a list of available commands, send `{} help`", msg.content, PREFIX, PREFIX);
                warn(ctx.send_message(&msg.channel_id, snd.as_str(), "", false));
                ctx
            })
        },
        Cmd {
            id: "test",
            cb: Box::new(|ctx: Discord, msg: &Message, args: Vec<&str>| -> Discord {
                let snd = format!("testing with closures {:?}", args);
                warn(ctx.send_message(&msg.channel_id, snd.as_str(), "", false));
                ctx
            }),
        },
    ];
    let (mut connection, ready) = discord.connect().expect("Connection failed.");
    println!("[Ready] {} is serving on {} server(s)", ready.user.username, ready.servers.len());
    let mut state = State::new(ready);
    connection.sync_calls(&state.all_private_channels());
    loop {
        let e = match connection.recv_event() {
            Ok(event) => event,
            Err(err) => {
                println!("[Warning] Received error: {:?}", err);
                if let discord::Error::WebSocket(..) = err {
                    let (new_connection, ready) = discord.connect().expect("Connection failed.");
                    connection = new_connection;
                    state = State::new(ready);
                    println!("[Ready] Reconnection successful.");
                }
                if let discord::Error::Closed(..) = err {
                    break
                }
                continue
            },
        };
        state.update(&e);
        match e {
            Event::MessageCreate(msg) => {
                if msg.author.id == state.user().id {
                    continue
                }
                let msg_s = msg.content.clone();
                let mut split = msg_s.split(' ');
                let first = split.next().unwrap_or("");
                if first != PREFIX {
                    continue
                }
                let cmd = split.next().unwrap_or("");
                let mut args: Vec<&str> = Default::default();
                for a in split { args.push(a); }
                for c in &commands {
                    if c.id == cmd {
                        discord = (c.cb)(discord, &msg.clone(), args.clone());
                        continue
                    }
                }
                // command not found
                discord = (commands[0].cb)(discord, &msg.clone(), args.clone());

            }
            _ => {},
        }
    }
}
fn warn<T, E: ::std::fmt::Debug>(result: Result<T, E>) {
    match result {
        Ok(_) => {},
        Err(err) => println!("[Warning] {:?}", err)
    }
}

