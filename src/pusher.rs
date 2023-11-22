use std::{thread::sleep, time::Duration};

use color_eyre::Result;
use tungstenite::{connect, Message};
use ureq::json;
use url::Url;

pub fn read_from_channel(channel: String) -> Result<()> {
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
                4a. Particularly important is knowing when to make tasks as done, and understanding if it needs to load up
                    new tasks down the chain.
    */
    let url =
        "ws://ws-us3.pusher.com:80/app/e6cb5e89997457f402f2?client=Wand&version=0.1.0&protocol=7";
    let (mut socket, _response) =
        connect(Url::parse(url).unwrap()).expect("Websocket connection error... :(");
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

    loop {
        let message = socket.read().expect("Couldn't read the message... :(");
        println!("> {}", message);
        sleep(Duration::from_secs(5));
    }
}
