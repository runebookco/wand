use std::{
    io::{stdout, Stdout, Write},
    net::TcpStream,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread::sleep,
    time::Duration,
};

use color_eyre::Result;
use colored::Colorize;
use crossterm::{
    cursor,
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use data_encoding::BASE64;
use serde::{Deserialize, Deserializer, Serialize};
use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};
use ureq::{
    json,
    serde_json::{self, Value},
};
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
struct PusherResponse {
    event: String,
    #[serde(deserialize_with = "deserialize_double_encoded_json_data")]
    data: Value,
}

struct Output {
    blocks: Vec<Block>,
}

impl Output {
    // TODO: I don't actually understand this and also it sometimes breaks (in my terminal) by double/triple printing
    // https://www.youtube.com/watch?v=qM4zMofsI7w
    fn print_output(&mut self, stdout: &mut Stdout, running: Arc<AtomicUsize>) {
        let mut output = String::new();

        for block in &mut self.blocks {
            output.push_str(block.as_terminal_output().as_str())
        }

        // The gist of what we are trying to do here is to have the output for steps sort of print and update
        // "in place"

        // So, we hide the blinking cursor
        stdout.execute(cursor::Hide).unwrap();
        // We store the location of the cursor and then return the cursor to the leftmost position in current line
        stdout.execute(cursor::SavePosition).unwrap();
        stdout.write_all("\r".as_bytes()).unwrap();
        // Then we send all buffered output to the window (why?) (is there even a buffer?????)
        // I don't know if flush moves the cursor.
        // Maybe the occasional double/weird output issues are related to the flush?
        stdout.flush().unwrap();
        // Then, assuming flush does not/has not moved the cursor, we clear from the start of current line down to the bottom
        stdout.execute(Clear(ClearType::FromCursorDown)).unwrap();
        // Then we rewrite the entire stored output
        // This whole process sort of emulates something akin to
        stdout.write_all(output.as_bytes()).unwrap();
        // Then we check if the user has hit ctrl+c and exit if so, other wise we restore our cursor to saved position
        if running.load(Ordering::SeqCst) > 0 {
            stdout.execute(cursor::Show).unwrap();
            std::process::exit(0);
        } else {
            stdout.execute(cursor::RestorePosition).unwrap();
        }
    }
}

#[derive(Debug)]
struct Block {
    name: String,
    body: String,
    step_num: u8,
    num_steps: u8,
    complete: bool,
}

impl Block {
    fn append_to_body(&mut self, string: &str) {
        self.body.push_str(string);
    }

    fn as_terminal_output(&self) -> String {
        format!(
            "{} [{}/{}] {}\n\n{}\n",
            match self.complete {
                true => "✓".green(),
                false => "🏃".into(),
            },
            self.step_num,
            self.num_steps,
            self.name.bright_white().bold(),
            self.body.truecolor(155, 155, 155)
        )
    }
}

// TODO: This feels more like parsing than deserializing, is there a better approach?
// Maybe https://crates.io/crates/nom ?
fn deserialize_double_encoded_json_data<'de, D>(deserializer: D) -> Result<Value, D::Error>
where
    D: Deserializer<'de>,
{
    let json_str = String::deserialize(deserializer)?;
    let json: Value = serde_json::from_str(&json_str).unwrap();
    match json["message"].is_string() {
        true => {
            let decoded_message = BASE64.decode(json["message"].as_str().unwrap().as_bytes());
            match decoded_message {
                Ok(bytes) => {
                    let message = &String::from_utf8(bytes).unwrap();
                    return Ok(json!({"message": message, "name": json["name"]}));
                }
                Err(_) => serde_json::from_str(&json_str).map_err(serde::de::Error::custom),
            }
        }
        false => serde_json::from_str(&json_str).map_err(serde::de::Error::custom),
    }
}

pub fn read_from_channel(channel: String, running: Arc<AtomicUsize>) -> Result<()> {
    /*
        For later:
            1. Reference python impl, looks sound: https://github.com/deepbrook/Pysher/blob/master/src/pysher/pusher.py
            2. Official pusher library implementation docs: https://pusher.com/docs/channels/library_auth_reference/auth-signatures/?ref=library_auth_reference
            3. There's a tokio-enabled version of tungstenite if we decide we need async
        TODO:
            1. Parameterize all the various pieces of the url here
            2. Read and fully(ish) implement the pusher spec (pusher:pongs etc.)
            3. Figure out auth for private channels
            4. Teach the reader how to respond to our protocol (after implementing our protocol)
                4a. Particularly important is knowing when to mark tasks as done, and understanding if it needs to load up
                    new tasks down the chain.
            5. Format the output in a way that is nice.
    */
    let url =
        "ws://ws-us3.pusher.com:80/app/e6cb5e89997457f402f2?client=Wand&version=0.1.0&protocol=7";
    let (mut socket, _response) = connect(Url::parse(url).unwrap())?;
    subscribe_to_channel(&mut socket, channel);
    let mut stdout = stdout();

    let output: &mut Output = &mut Output { blocks: Vec::new() };

    loop {
        let message = socket.read().expect("Couldn't read the message... :(");

        // TODO: This feels unwieldy, can likely be broken down
        match message {
            Message::Text(_) => {
                let json_message: PusherResponse = serde_json::from_str(message.to_text()?)
                    .expect("Failed to parse Pusher Response... :(");
                match json_message.event.as_str() {
                    "pusher:connection_established" => {}
                    "pusher_internal:subscription_succeeded" => {}
                    "message" => {
                        // TODO: Need some way to understand we are in a new `Block`
                        // For now, just assume 1️⃣
                        match output.blocks.last_mut() {
                            None => {
                                // Will need to establish context containing the current step & num of steps outside of
                                // this loop so that it can be available going fwd but we don't yet have that info
                                // so this is fine for demo purposes.
                                output.blocks.push(Block {
                                    name: "Test first one".to_string(),
                                    body: "Test body here \nHere is some more output: 1\nAnd more: 2\n".to_string(),
                                    step_num: 1,
                                    num_steps: 2,
                                    complete: true,
                                });

                                output.blocks.push(Block {
                                    // TODO: There's gotta be a better way to handle this lmao
                                    // just doing "to_string()" makes it print with quotes around it...
                                    name: json_message.data["name"].as_str().unwrap().to_string(),
                                    body: json_message.data["message"]
                                        .as_str()
                                        .unwrap()
                                        .to_string(),
                                    step_num: 2,
                                    num_steps: 2,
                                    complete: false,
                                });
                            }
                            Some(block) => {
                                // Already have our 1 block, append to it and redraw
                                block.append_to_body(
                                    &json_message.data["message"].as_str().unwrap().to_string(),
                                );
                            }
                        }

                        output.print_output(&mut stdout, running.clone());
                    }
                    &_ => {
                        println!("Unknown event type: {:?}", json_message);
                    }
                }
            }
            _ => {
                println!("Received unaccounted for message type: {}", message);
            }
        }
        sleep(Duration::from_millis(500));
    }
}

pub fn subscribe_to_channel(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>, channel: String) {
    let subscribe_event = json!(
        {
            "event": "pusher:subscribe",
            "data": {
                "channel": channel,
            }
        }
    );
    socket
        .send(Message::Text(subscribe_event.to_string()))
        .expect("Couldn't subscribe to channel... :(");
}
