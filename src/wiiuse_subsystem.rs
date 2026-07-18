use std::{
    sync::mpsc::{self, Receiver, channel},
    thread::{self, JoinHandle},
};

pub fn start_wiiuse_subsystem() -> (JoinHandle<()>, Receiver<i32>) {
    let (tx, rx) = channel();
    let jhd = thread::spawn(move || {
        tx.send(10).unwrap();
    });

    return (jhd, rx);
}
