use std::collections::HashMap;
use std::io::{BufRead, BufReader, LineWriter, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

pub struct Engine {
    pub path: String,
    pub uci_options: HashMap<String, String>,
}

pub fn read_line<T: BufRead>(buf: &mut T) -> String {
    let mut res = String::new();
    buf.read_line(&mut res).unwrap();
    res
}

impl Engine {
    pub fn from_path(path: &str, uci_options: HashMap<String, String>) -> Engine {
        Engine {
            path: path.to_string(),
            uci_options,
        }
    }

    pub fn request(
        &mut self,
        fen: &str,
        go_string: &str,
        stdin: &mut LineWriter<ChildStdin>,
        stdout: &mut BufReader<ChildStdout>,
    ) -> String {
        writeln!(stdin, "position fen {}", fen).unwrap();
        writeln!(stdin, "go {}", go_string).unwrap();
        let mut res = String::new();
        loop {
            let engine_output = read_line(stdout);
            if engine_output.contains("bestmove") {
                return res;
            }
            res = engine_output;
        }
    }

    pub fn initialize_engine(
        &self,
        stdin: &mut LineWriter<ChildStdin>,
        stdout: &mut BufReader<ChildStdout>,
    ) {
        writeln!(stdin, "uci").unwrap();
        while !read_line(stdout).contains("uciok") {}
        for pair in &self.uci_options {
            writeln!(stdin, "setoption name {} value {}", pair.0, pair.1).unwrap();
        }
    }

    pub fn get_handles(&self) -> (Child, LineWriter<ChildStdin>, BufReader<ChildStdout>) {
        let mut cmd = Command::new(&self.path);
        cmd.stdin(Stdio::piped()).stdout(Stdio::piped());
        let mut child = cmd.spawn().unwrap();
        let stdin = LineWriter::new(child.stdin.take().unwrap());
        let stdout = BufReader::new(child.stdout.take().unwrap());
        (child, stdin, stdout)
    }
}
