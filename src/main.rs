use clap::Parser;
use std::{
    io::Write,
    sync::Arc,
    time::{SystemTime, SystemTimeError},
};
use tokio::sync::Mutex;

#[derive(Parser)]
struct Args {
    url: String,
    /** Interrupt when success */
    #[arg(short)]
    interrupt: bool,
    /** Times to request */
    #[arg(short, long)]
    times: Option<usize>,
    /** Request jobs */
    #[arg(short = 'j', long, default_value_t = 1)]
    jobs: usize,
}

#[inline(always)]
fn get_reqps(reqs: usize, time_interval: (SystemTime, SystemTime)) -> Result<f64, SystemTimeError> {
    Ok(reqs as f64 * 1_000_000.
        / time_interval.1.duration_since(time_interval.0)?.as_micros() as f64)
}

struct Progress {
    count: usize,
    count_interval: (usize, usize),
    time_interval: (SystemTime, SystemTime),
}

impl Progress {
    fn new() -> Self {
        Self {
            count: 0,
            count_interval: (0, 0),
            time_interval: (SystemTime::now(), SystemTime::now()),
        }
    }

    async fn refresh(&mut self, times: Option<usize>) -> anyhow::Result<()> {
        self.count += 1;

        if let Some(times) = times {
            print!(
                "\r[{:7}/{}] {:10.2} req/s",
                self.count,
                times,
                get_reqps(
                    self.count_interval.1 - self.count_interval.0,
                    self.time_interval
                )?
            );
        } else {
            print!(
                "\r[{:7}/unlimited] {:10.2} req/s",
                self.count,
                get_reqps(
                    self.count_interval.1 - self.count_interval.0,
                    self.time_interval
                )?
            );
        }
        std::io::stdout().flush()?;
        if SystemTime::now()
            .duration_since(self.time_interval.1)?
            .as_secs()
            >= 1
        {
            self.time_interval.0 = self.time_interval.1;
            self.count_interval.0 = self.count_interval.1;
            self.time_interval.1 = SystemTime::now();
            self.count_interval.1 = self.count;
        }

        Ok(())
    }
}

struct RequestJob {
    url: String,
    t: Arc<Mutex<Progress>>,
    interrupt: bool,
    times: Option<usize>,
}

impl RequestJob {
    async fn start_request(self) -> anyhow::Result<()> {
        loop {
            self.t.lock().await.refresh(self.times).await?;

            let response = reqwest::get(&self.url).await;

            if response.is_ok() && self.interrupt {
                break;
            }

            let mut t = self.t.lock().await;
            t.count += 1;
            if let Some(times) = self.times
                && t.count == times
            {
                break;
            }
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let t = Arc::new(Mutex::new(Progress::new()));
    let mut tasks = Vec::new();
    for _ in 0..args.jobs {
        let job = RequestJob {
            url: args.url.clone(),
            t: Arc::clone(&t),
            interrupt: args.interrupt,
            times: args.times,
        };
        tasks.push(job.start_request());
    }
    futures::future::join_all(tasks).await;
    println!();

    Ok(())
}
