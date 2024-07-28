use std::{
    io::{Read, Write},
    net::TcpListener,
};

use build_your_own_utils::my_own_error::MyOwnError;

// https://codingchallenges.fyi/challenges/challenge-redis

pub fn redis_cli(_args: &[&str]) -> Result<(), MyOwnError> {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        let mut stream = stream?;

        let mut buffer = [0; 1024];
        stream.read(&mut buffer)?;
        println!("Request: {}", String::from_utf8_lossy(&buffer[..]));

        // TODO: parse request
        // TODO: calculate response

        let response = b"+OK\r\n";
        stream.write(response)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {}
