use anyhow::Result;
use async_trait::async_trait;
use std::time::SystemTime;
use std::{pin::Pin, time::Duration};
use tokio::time::Sleep;

#[async_trait]
trait State {
    async fn timeout(&mut self) -> Result<()>;
    fn extend_timeout(&mut self, duration: u64);
}

/// Wrapper struct to hold the state. This is useful when we want to hold a dyn trait object 
struct Wrapper<S> {
    state: S,
}

struct GameInitState {
    timeout: u64,
    sleep: Option<Pin<Box<Sleep>>>,
}

#[async_trait]
impl State for Wrapper<GameInitState> {
    async fn timeout(&mut self) -> Result<()> {
        let sleep = self.state.sleep.as_mut().unwrap();
        sleep.await;
        Ok(())
    }

    fn extend_timeout(&mut self, duration: u64) {
        let deadline = self.state.sleep.as_ref().unwrap().deadline();
        let mut sleep = self.state.sleep.take().unwrap();
        sleep
            .as_mut()
            .reset(deadline + Duration::from_secs(duration));
        self.state.sleep = Some(sleep);
    }
}
impl Wrapper<GameInitState> {
    fn new(timeout: u64) -> Self{
        let m = tokio::time::sleep(Duration::from_secs(timeout));
        Self{
            state: GameInitState{
                timeout,
                sleep: Some(Box::pin(m)),
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let start = SystemTime::now();
    let mut state: Box<dyn State> = Box::new(Wrapper::<GameInitState>::new(5));


    let (tx, mut rx) = tokio::sync::mpsc::channel::<u64>(100);

    tokio::spawn(async move {
        tx.send(10).await.unwrap();
    });
    loop{
        tokio::select! {
            res = rx.recv() => {
                if let Some(res) = res{
                    state.extend_timeout(res);
                }
            }
            _ = state.timeout() => {
                println!("Time out: {:?}", start.elapsed().unwrap().as_secs());
                break;
            }
        }
    }

}
