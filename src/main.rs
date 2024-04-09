use tokio::sync::{mpsc, oneshot, mpsc::Sender};

#[derive(Debug, Clone)]
pub enum Order {
    BUY,
    SELL
}

#[derive(Debug)]
pub struct Message {
    pub order: Order,
    pub ticker: String,
    pub amount: f32,
    pub respond_to: oneshot::Sender<u32>
}

pub struct OrderBookActor {
    pub receiver: mpsc::Receiver<Message>,
    pub total_invested: f32,
    pub investment_cap: f32
}

impl OrderBookActor {
    fn new(receiver: mpsc::Receiver<Message>, investment_cap: f32) -> Self {
        Self { receiver, total_invested: 0.0, investment_cap }
    }

    fn handle_message(&mut self, message: Message) {
        if message.amount + self.total_invested >= self.investment_cap {
            println!("rejecting purchase, total invested: {}", self.total_invested);
            let _ = message.respond_to.send(0);
        } else {
            self.total_invested += message.amount;
            println!("processing purchase, total invested: {}", self.total_invested);
            let _ = message.respond_to.send(1);
        }
    }

    async fn run(mut self) {
        println!("actor is running");
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg);
        }
    }
}

struct BuyOrder {
    pub order: Order,
    pub ticker: String,
    pub amount: f32,
    pub sender: Sender<Message>
}

impl BuyOrder {
    fn new(ticker: String, amount: f32, sender: Sender<Message>) -> Self {
        Self { order: Order::BUY, ticker, amount, sender }
    }

    async fn send(self) {
        let (send, recv) = oneshot::channel();
        let message = Message {
            order: self.order,
            amount: self.amount,
            ticker: self.ticker,
            respond_to: send
        };
        let _ = self.sender.send(message).await;
        match recv.await {
            Ok(outcome) => println!("here is the outcome: {}", outcome),
            Err(e) => println!("{}", e)
        }
    }
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<Message>(1);
    let tx_one = tx.clone();

    tokio::spawn(async move {
        for _ in 0..5 {
            let but_actor = BuyOrder::new("BYND".to_owned(), 5.5, tx_one.clone());
            but_actor.send().await;
        }
        drop(tx_one);
    });
    tokio::spawn(async move {
        for _ in 0..5 {
            let but_actor = BuyOrder::new("PLTR".to_owned(), 5.5, tx.clone());
            but_actor.send().await;
        }
        drop(tx);
    });

    let actor = OrderBookActor::new(rx, 20.0);
    actor.run().await;
}
