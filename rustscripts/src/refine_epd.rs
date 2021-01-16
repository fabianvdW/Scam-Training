use shakmaty::fen::{self, Fen};
use shakmaty::{Chess, Position};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::sync::{Arc, Mutex};
use std::thread::spawn;

use rustscripts::engine::*;
use shakmaty::uci::Uci;
use std::collections::HashMap;
use std::str::FromStr;

//A script for parsing files of Andrews.epd format into our csv batch format.
fn main() {
    let args: Vec<String> = env::args().collect();
    let mut in_file: Option<String> = None;
    let mut o_file = None;
    let mut engine: Option<String> = None;
    let mut threads: usize = 1;
    let mut depth = 6;
    let mut filterlimit = 1000;
    for i in 0..args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "-file" => in_file = Some(args[i + 1].parse().unwrap()),
            "-ofile" => o_file = Some(args[i + 1].parse().unwrap()),
            "-threads" => threads = args[i + 1].parse().unwrap(),
            "-engine" => engine = Some(args[i + 1].parse().unwrap()),
            "-depth" => depth = args[i + 1].parse().unwrap(),
            "-filterlimit" => filterlimit = args[i + 1].parse().unwrap(),
            _ => {}
        }
    }
    if o_file.is_none() {
        assert!(in_file.as_ref().unwrap().contains(".epd"));
        o_file = Some(in_file.as_ref().unwrap().replace(".epd", "_refined.epd"));
    }

    let infile = File::open(in_file.unwrap()).unwrap();
    let infile = BufReader::new(infile);

    let outfile = Arc::new(Mutex::new(BufWriter::new(
        File::create(o_file.unwrap()).unwrap(),
    )));

    let mut work_queue = Vec::new();
    for line in infile.lines() {
        let line = line.expect("Unable to read line");
        let mut iter = line.split('[');
        let fen = iter.next().unwrap().trim_end();
        let score = iter.next().unwrap().replace("]", "");
        work_queue.push((fen.to_owned(), score));
    }
    println!("Prepared work queue with {} elements!", work_queue.len());

    let work_queue = Arc::new(Mutex::new(work_queue));
    let mut workers = Vec::new();

    for _ in 0..threads {
        let work_queue = Arc::clone(&work_queue);
        let out_writer = Arc::clone(&outfile);
        let mut options = HashMap::new();
        options.insert("Threads".to_owned(), "1".to_owned());
        options.insert("Hash".to_owned(), "16".to_owned());
        let mut engine = Engine::from_path(engine.as_ref().unwrap(), options);

        workers.push(spawn(move || {
            let (mut child, mut child_in, mut child_out) = engine.get_handles();
            engine.initialize_engine(&mut child_in, &mut child_out);

            let mut local_queue = Vec::new();
            let mut local_res_queue = Vec::new();
            loop {
                local_queue.clear();
                local_res_queue.clear();
                fetch_queue(&mut local_queue, &work_queue);
                if local_queue.len() == 0 {
                    break;
                }
                while let Some((fen, result)) = local_queue.pop() {
                    let latest_pv = engine.request(
                        &fen,
                        &format!("depth {}", depth),
                        &mut child_in,
                        &mut child_out,
                    );
                    if latest_pv.contains("mate") || !latest_pv.contains("score") {
                        continue;
                    }
                    let score: i32 = latest_pv
                        .rsplit("score cp ")
                        .next()
                        .unwrap()
                        .split_whitespace()
                        .next()
                        .unwrap()
                        .parse()
                        .unwrap();
                    if score.abs() > filterlimit {
                        continue;
                    }
                    let latest_pv = latest_pv.rsplit("pv ").next().unwrap().trim_end();

                    let mut position: Chess = Fen::from_str(&fen).unwrap().position().unwrap();
                    for mv in latest_pv.split(' ') {
                        let mv = Uci::from_str(mv)
                            .ok()
                            .and_then(|x| x.to_move(&position).ok())
                            .unwrap();
                        position = position.play(&mv).unwrap();
                    }
                    local_res_queue.push(format!("{}, [{}]", fen::fen(&position), result));
                }
                let mut write = out_writer.lock().unwrap();
                for s in local_res_queue.iter() {
                    writeln!(&mut write, "{}", s).unwrap();
                }
                std::mem::drop(write);
            }
            child.kill().unwrap();
        }));
    }

    for worker in workers.into_iter() {
        worker.join().unwrap();
    }
    outfile.lock().unwrap().flush().unwrap();
}
fn fetch_queue<T>(local_queue: &mut Vec<T>, worker_queue: &Arc<Mutex<Vec<T>>>) {
    let mut worker_queue = worker_queue.lock().unwrap();
    for _ in 0..100 {
        if let Some(t) = worker_queue.pop() {
            local_queue.push(t);
        }
    }
}
