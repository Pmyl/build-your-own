use build_your_own_utils::my_own_error::MyOwnError;
use std::collections::HashMap;
use std::{
    io::{Read, Write},
    net::TcpListener,
};

// https://codingchallenges.fyi/challenges/challenge-redis

pub fn redis_cli(_args: &[&str]) -> Result<(), MyOwnError> {
    let port = 6379;
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
    println!("Listening on port {}", port);

    let mut redis = Redis::new();

    for stream in listener.incoming() {
        let mut stream = stream?;

        let mut buffer = [0; 1024];
        stream.read(&mut buffer)?;
        let request = String::from_utf8_lossy(&buffer);

        let response = redis.process(&request);
        stream.write(response.as_bytes())?;
    }

    Ok(())
}

struct Redis {
    data: HashMap<String, String>,
}

impl Redis {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
    
    fn process(&mut self, input: &str) -> String {
        let arguments = parse_input(input);

        let first_argument = arguments[0];

        match first_argument {
            "ECHO" => {
                if arguments.len() != 2 {
                    "-ERR wrong number of arguments for command\r\n".to_string()
                } else {
                    format!("+{}\r\n", arguments[1])
                }
            }
            "PING" => "+PONG\r\n".to_string(),
            "SET" => {
                self.data
                    .insert(arguments[1].to_string(), arguments[2].to_string());
                "+OK\r\n".to_string()
            }
            "GET" => {
                let value = self.data.get(&arguments[1].to_string());
                match value {
                    None => "$-1\r\n".to_string(),
                    Some(value) => format!("+{}\r\n", value),
                }
            }
            _ => format!("-unknown command '{}'\r\n", first_argument),
        }
    }
}

fn parse_input(input: &str) -> Vec<&str> {
    let mut result = vec![];

    input.split("\r\n").skip(1).enumerate().for_each(|(i, x)| {
        if i % 2 == 1 {
            result.push(x);
        }
    });

    return result;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn parse_input_test() {
        let result = parse_input("*1\r\n$4\r\nPING\r\n");
        assert_eq!(result, vec!["PING"]);

        let result = parse_input("*2\r\n$4\r\nECHO\r\n$11\r\nHello World");
        assert_eq!(result, vec!["ECHO", "Hello World"]);
    }
    
    #[test]
    fn pong() {
        let mut redis = Redis::new();

        let result = redis.process("*1\r\n$4\r\nPING");
        assert_eq!(result, "+PONG\r\n");
    }

    #[test]
    fn set() {
        let mut redis = Redis::new();
        let result = redis.process("*3\r\n$3\r\nSET\r\n$4\r\nName\r\n$4\r\nJohn\r\n");
        assert_eq!(result, "+OK\r\n");

        let result = redis.process("*2\r\n$3\r\nGET\r\n$4\r\nName\r\n");

        assert_eq!(result, "+John\r\n");
    }

    #[test]
    fn get_missing() {
        let mut redis = Redis::new();

        let result = redis.process("*2\r\n$3\r\nGET\r\n$4\r\nName\r\n");

        assert_eq!(result, "$-1\r\n");
    }

    #[test]
    fn echo() {
        let mut redis = Redis::new();
        let result = redis.process("*2\r\n$4\r\nECHO\r\n$11\r\nHello World");
        assert_eq!(result, "+Hello World\r\n");
    }

    #[test]
    fn echo_missing_arguments() {
        let mut redis = Redis::new();
        let result = redis.process("*1\r\n$4\r\nECHO\r\n");
        assert_eq!(result, "-ERR wrong number of arguments for command\r\n");
    }

    #[test]
    fn echo_too_many_arguments() {
        let mut redis = Redis::new();
        let result = redis.process("*3\r\n$4\r\nECHO\r\n$1\r\nN\r\n$1\r\nB\r\n");
        assert_eq!(result, "-ERR wrong number of arguments for command\r\n");
    }

    #[test]
    fn unknown_command() {
        let mut redis = Redis::new();
        let result = redis.process("*1\r\n$4\r\nCIAO\r\n");
        assert_eq!(result, "-unknown command 'CIAO'\r\n");
    }
}
