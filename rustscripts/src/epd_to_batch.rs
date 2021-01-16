use std::env;
use std::fs::{read_dir, File};
use std::io::{BufRead, BufReader, LineWriter, Write};
use std::path::Path;

//A script for parsing files of Andrews.epd format into our csv batch format.
fn main() {
    let args: Vec<String> = env::args().collect();
    let mut batch_size = 1_000_000;
    let mut convert_path: Option<String> = None;
    let mut out_folder = "./batches/".to_owned();
    for i in 0..args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "-batchsize" => batch_size = args[i + 1].parse().unwrap(),
            "-file" => convert_path = Some(args[i + 1].parse().unwrap()),
            "-ofolder" => out_folder = args[i + 1].parse().unwrap(),
            _ => {}
        }
    }

    let infile = File::open(convert_path.unwrap()).expect("Unable to open input file");
    let infile = BufReader::new(infile);

    let mut current_highest_batch = -1;
    for file in read_dir(&out_folder).unwrap() {
        let path_to_file = file.unwrap().path();
        let path_to_file = path_to_file.to_str().unwrap();
        let batch_num = path_to_file
            .split(".csv")
            .next()
            .unwrap()
            .rsplit('_')
            .next()
            .unwrap();
        if let Ok(num) = batch_num.parse() {
            current_highest_batch = current_highest_batch.max(num);
        }
    }
    current_highest_batch += 1;

    let mut outfile = new_file(&out_folder, &mut current_highest_batch);
    let mut current_batch = 0;

    for line in infile.lines() {
        let line = line.expect("Unable to read line");
        let mut iter = line.split('[');
        let fen = iter.next().unwrap();
        let score = iter
            .next()
            .unwrap()
            .replace("]", "")
            .parse::<f32>()
            .unwrap();
        writeln!(&mut outfile, "{},{}", fen, score).unwrap();
        current_batch += 1;
        if current_batch % batch_size == 0 {
            current_batch = 0;
            outfile.flush().unwrap();
            outfile = new_file(&out_folder, &mut current_highest_batch);
        }
    }
    outfile.flush().unwrap();
}

fn new_file(out_folder: &str, batch: &mut i32) -> LineWriter<File> {
    let path = Path::new(out_folder).join(format!("batch_{}.csv", *batch));
    let outfile = File::create(path).expect("Unable to create output file");
    let mut outfile = LineWriter::new(outfile);
    writeln!(&mut outfile, "Fen,Result").unwrap();
    *batch += 1;
    outfile
}
