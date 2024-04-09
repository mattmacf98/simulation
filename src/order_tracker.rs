use tokio::sync::{mpsc, oneshot};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Order {
    BUY(String, f32),
    GET
}

pub struct TrackerMessage {
    pub command: Order,
    pub respond_to: oneshot::Sender<String >
}

pub struct GetTrackerActor {
    pub sender: mpsc::Sender<TrackerMessage>
}

impl GetTrackerActor {
    pub async fn send(self) -> String {
        println!("GET function firing");
        let (send, recv) = oneshot::channel();
        let tracker_message = TrackerMessage {
            command: Order::GET,
            respond_to: send,
        };

        let _ = self.sender.send(tracker_message).await;
        match recv.await {
            Ok(outcome) => return outcome,
            Err(e) => panic!("{}", e)
        }
    }
}

pub struct TrackerActor {
    pub receiver: mpsc::Receiver<TrackerMessage>,
    pub db: HashMap<String, f32>
}

impl TrackerActor {
    pub fn new(receiver: mpsc::Receiver<TrackerMessage>) -> Self {
        Self { receiver, db: HashMap::new() }
    }

    fn send_state(&self, respond_to: oneshot::Sender<String>) {
        let mut buffer = Vec::new();
        for key in self.db.keys() {
            let amount = self.db.get(key).unwrap();
            buffer.push(format!("{}: {};", &key, amount));
        }
        buffer.push("\n".to_string());
        println!("sending state: {}", buffer.join(""));
        let _ = respond_to.send(buffer.join(""));
    }

    fn handle_message(&mut self, message: TrackerMessage) {
        match message.command {
            Order::BUY(ticker, amount) => {
                match self.db.get(ticker.as_str()) {
                    None => self.db.insert(ticker, amount),
                    Some(val) => self.db.insert(ticker, amount + val)
                };
                println!("db: {:?}", self.db);
            }
            Order::GET => {
                println!("getting state");
                self.send_state(message.respond_to);
            }
        };
    }

    pub async fn run(mut self) {
        println!("tracker actor is running");
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg);
        }
    }
}
