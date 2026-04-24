use std::{thread, time::{Duration, SystemTime, UNIX_EPOCH}};

use crossbeam::channel::{Receiver, Sender, TryRecvError};

use crate::archive::{Fragment, FragmentBody};

#[derive(Debug)]
pub struct Config {
    pub cycle_time: Duration,
}

#[derive(Debug)]
pub struct Recorder<const N: usize, Body: FragmentBody<N>> {
    config: Config,
    record_rx: Receiver<Body::Record>, 
    fragment_tx: Sender<Fragment<N, Body>>
}

impl<const N: usize, Body: FragmentBody<N>> Recorder<N, Body> {
    pub fn new(
        config: Config, 
        record_rx: Receiver<Body::Record>, 
        fragment_tx: Sender<Fragment<N, Body>>
    ) -> Self {
        Self { config, record_rx, fragment_tx }
    }

    pub fn run(self) {
        let cycle_time = self.config.cycle_time.as_micros() as u64;

        let mut fragment_ts = Self::get_timestamp();
        let mut fragment    = Fragment::new(fragment_ts); 

        loop {
            let now = Self::get_timestamp();
            let dt  = now - fragment_ts;   

            if dt >= cycle_time { 
                fragment_ts = now;

                let old_fragment = std::mem::replace(
                    &mut fragment,
                    Fragment::new(fragment_ts),
                );

                println!("Fragment complete: {:?}", fragment_ts);

                let segment_tx = self.fragment_tx.clone();
                thread::spawn(move || {
                    // let thread block until channel is open
                    let res = segment_tx.send(old_fragment);
                    res.expect("Failed to submit");
                });
            }

            loop {
                match self.record_rx.try_recv() {
                    Ok(record) => {
                        fragment.append(now, record);
                    },
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        panic!("MainThread Channel Disconnected");
                    }
                }
            }
        }
    }

    fn get_timestamp() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).expect("No overflows").as_micros() as u64
    }
}