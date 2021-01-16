use pgn_reader::{Reader, San, Skip, Visitor};
use rand::prelude::ThreadRng;
use rand::seq::IteratorRandom;
use shakmaty::fen::{fen, Fen};
use shakmaty::{Chess, Position};
use std::env;
use std::fs::File;
use std::io::{BufReader, LineWriter, Read, Write};

struct FenSampler<T: Write> {
    pub out: T,
    pub samples_per_game: usize,
    pub rng: ThreadRng,

    pub pos: Chess,
    pub plycount: usize,
    pub sample_indices: Vec<usize>,
    pub result: &'static str,
    pub skip: bool,
}

impl<T: Write> FenSampler<T> {
    pub fn new(out: T, samples_per_game: usize) -> FenSampler<T> {
        FenSampler {
            sample_indices: Vec::new(),
            result: "",
            skip: false,
            out,
            pos: Chess::default(),
            plycount: 0,
            rng: ThreadRng::default(),
            samples_per_game,
        }
    }
    pub fn log(&mut self) {
        writeln!(&mut self.out, "{} {}", fen(&self.pos), self.result).unwrap();
    }
}

impl<'pgn, T: Write> Visitor<'pgn> for FenSampler<T> {
    type Result = ();

    fn begin_game(&mut self) {
        self.sample_indices.clear();
        self.plycount = 0;
        self.skip = false;
        self.pos = Chess::default();
    }

    fn header(&mut self, key: &'pgn [u8], value: &'pgn [u8]) {
        if key == b"FEN" {
            let pos = Fen::from_ascii(value).ok().and_then(|f| f.position().ok());
            if let Some(pos) = pos {
                self.pos = pos;
            }
        } else if key == b"Result" {
            if value == b"0-1" {
                self.result = "[0.0]";
            } else if value == b"1-0" {
                self.result = "[1.0]"
            } else if value == b"1/2-1/2" {
                self.result = "[0.5]"
            } else {
                self.skip = true;
                assert_eq!(value, b"*");
            }
        } else if key == b"Termination" && value != b"normal" && value != b"adjudication" {
            self.skip = true;
        } else if key == b"PlyCount" {
            let plies = std::str::from_utf8(value)
                .unwrap()
                .parse::<usize>()
                .unwrap();
            self.sample_indices = (0..plies).choose_multiple(&mut self.rng, self.samples_per_game);
        }
    }

    fn end_headers(&mut self) -> Skip {
        Skip(self.skip)
    }

    fn san(&mut self, san: San) {
        if self.sample_indices.contains(&self.plycount) {
            self.log();
        }
        let m = san.to_move(&self.pos).unwrap();
        self.pos.play_unchecked(&m);
        self.plycount += 1;
    }

    fn begin_variation(&mut self) -> Skip {
        Skip(true)
    }

    fn end_game(&mut self, _game: &'pgn [u8]) -> Self::Result {}
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut samples_per_game = 15;

    let mut convert_path: Option<String> = None;
    let mut out_file: Option<String> = None;
    for i in 0..args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "-file" => convert_path = Some(args[i + 1].parse().unwrap()),
            "-ofile" => out_file = Some(args[i + 1].parse().unwrap()),
            "-samples" => samples_per_game = args[i + 1].parse().unwrap(),
            _ => {}
        }
    }
    if out_file.is_none() {
        assert!(convert_path.as_ref().unwrap().contains(".pgn"));
        out_file = Some(convert_path.as_ref().unwrap().replace(".pgn", ".epd"));
    }

    let infile = File::open(convert_path.unwrap()).expect("Unable to open input file");
    let mut buf = Vec::new();
    let pgns = BufReader::new(infile).read_to_end(&mut buf);
    println!("Read {} bytes!", pgns.unwrap());

    let outfile = File::create(out_file.unwrap()).unwrap();
    let outfile = LineWriter::new(outfile);

    let mut fensampler = FenSampler::new(outfile, samples_per_game);
    let mut reader = Reader::new(&mut fensampler, &buf);
    let mut games = 0;
    while let Some(_) = reader.read_game() {
        games += 1;
    }
    println!("Sampled {} games!", games);
}
