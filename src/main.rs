extern crate discord;
use discord::{Discord, State};
use discord::model::{Event, Message};
use std::env;
use std::fmt::{Display, Formatter, Error};
static PREFIX: &'static str = "*69";

struct Cmd {
    id: &'static str,
    cb: Box<Fn(Discord, &Message, Vec<&str>) -> Discord>,
    desc: &'static str,
}
impl Display for Cmd {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "`{}`:\n\t{}", &self.id, &self.desc)
    }
}
impl Clone for Cmd {
    fn clone(&self) -> Cmd {
        Cmd {
            id: &self.id.clone(),
            cb: Box::new(move |ctx: Discord, msg: &Message, args: Vec<&str>| -> Discord {ctx}),
            desc: &self.id.clone()
        }
    }
}
impl PartialEq for Cmd {
    fn eq(&self, other: &Cmd) -> bool {
        self.id == other.id
    }
}
fn init_commands() -> Vec<Cmd> {
    let mut commands: Vec<Cmd> = vec![
        Cmd {
            id: "_",
            cb: Box::new(|ctx: Discord, msg: &Message, args: Vec<&str>| -> Discord {
                let snd = format!("Unknown command `{}`\nUsage: `{} <cmd> <args>`\nFor a list of available commands, send `{} help`", msg.content, PREFIX, PREFIX);
                warn(ctx.send_message(&msg.channel_id, snd.as_str(), "", false));
                ctx
            }),
            desc: ""
        },
        Cmd {
            id: "test",
            cb: Box::new(|ctx: Discord, msg: &Message, args: Vec<&str>| -> Discord {
                let snd = format!("testing with closures {:?}", args);
                warn(ctx.send_message(&msg.channel_id, snd.as_str(), "", false));
                ctx
            }),
            desc: "A test command"
        },
    ];
    let tmp: Vec<(&str, &str)> = commands.iter().map(|x| (x.id, x.desc)).collect();
    commands.push(
        Cmd {
            id: "help",
            cb: Box::new(move |ctx: Discord, msg: &Message, args: Vec<&str>| -> Discord {
                let mut out = String::new();
                out.push_str("List of available commands:\n");
                for cmd in &tmp {
                    if cmd.0 == "_" {
                        continue
                    }
                    out.push_str(format!("`{}:`\n\t{}\n", cmd.0, cmd.1).as_str());
                }
                warn(ctx.send_message(&msg.channel_id, out.as_str(), "", false));
                ctx
            }),
            desc: "List all commands"
        }
    );
    commands
}
fn main() {
    let mut discord = Discord::from_bot_token(&env::var("DISCORD_TOKEN").expect("Could not find token."))
        .expect("Login failed.");
    let commands = init_commands();
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
                if commands.iter().find(|ref x| x.id == cmd) == None {
                    // command not found
                    discord = (commands[0].cb)(discord, &msg.clone(), args.clone());
                }
                for c in &commands {
                    if c.id == cmd {
                        discord = (c.cb)(discord, &msg.clone(), args.clone());
                        continue
                    }
                }

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

