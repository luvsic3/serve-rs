use std::{
    env, fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex, MutexGuard},
};

use serve_rs::ThreadPool;

const GET: &str = "GET";
const SUCCESS: &str = "HTTP/1.1 200 OK";
const NOT_FOUND: &str = "HTTP/1.1 404 NOT FOUND";

fn main() {
    let args: Vec<String> = env::args().collect();
    let options = parse_options(&args);
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let thread_pool = ThreadPool::new(10);
    let options_arc = Arc::new(Mutex::new(options));
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let options_arc = Arc::clone(&options_arc);
        thread_pool.execute(move || {
            let options = options_arc.lock().unwrap();
            handle_connection(stream, options);
        })
    }
}

fn handle_connection(mut stream: TcpStream, options: MutexGuard<Options>) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let request_str = String::from_utf8_lossy(&buffer).to_string();
    let request_line = request_str.lines().next().unwrap();
    let request_characters: Vec<&str> = request_line.split(" ").collect();
    let method = request_characters[0];
    let uri = request_characters[1];
    let (status_line, contents) = if GET.eq(method) {
        let file_path = format!("{}{}", options.path, uri);
        let contents = fs::read_to_string(file_path);
        match contents {
            Ok(con) => (SUCCESS, con),
            Err(_) => (NOT_FOUND, "".to_string()),
        }
    } else {
        (NOT_FOUND, "".to_string())
    };
    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

struct Options {
    path: String,
}

fn parse_options(strs: &Vec<String>) -> Options {
    let path = strs[1].clone();
    Options { path: path.clone() }
}
