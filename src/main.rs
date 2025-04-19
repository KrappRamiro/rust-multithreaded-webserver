use hello::ThreadPool;
use std::{
    // We bring std::io::prelude and std::io::BufReader into scope to get access to traits and types that let us read from and write to the stream
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming().take(2) {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    // BufReader adds buffering by managing calls to the std::io::Read trait methods for us.
    // BufReader implements the std::io::BufRead trait, which provides the .lines() method.
    // The .lines() method returns an iterator of `Result<String, std::io::Error>` by splitting the stream of data whenever it sees a newline byte.
    // To get each String, we map and unwrap each Result. The Result might be an error if the data isn’t valid UTF-8 or if there was a problem reading from the stream.
    // Again, a production program should handle these errors more gracefully, but we’re choosing to stop the program in the error case for simplicity
    let buf_reader = BufReader::new(&stream);
    // We create a variable named http_request to collect the lines of the request the browser sends to our server
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    // We need to explicitly match on a slice of request_line to pattern match against the string literal values
    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    stream.write_all(response.as_bytes()).unwrap();
}
