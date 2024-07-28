use std::{
    io::{Read, Write},
    net::TcpListener,
};
use std::collections::HashMap;
use build_your_own_utils::my_own_error::MyOwnError;

// https://codingchallenges.fyi/challenges/challenge-redis

pub fn redis_cli(_args: &[&str]) -> Result<(), MyOwnError> {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        let mut stream = stream?;

        let mut buffer = [0; 1024];
        stream.read(&mut buffer)?;
        let request = String::from_utf8_lossy(&buffer);
        println!("Request: {}", request);

        let response = redis(&request, &mut HashMap::new());

        println!("Response {}", response);

        stream.write(response.as_bytes())?;
    }

    Ok(())
}

fn redis(input: &str, data: &mut HashMap<String, String>) -> String {
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
        "PING" => {
            "+PONG\r\n".to_string()
        }
        "SET" => {
            data.insert(arguments[1].to_string(), arguments[2].to_string());
            "+OK\r\n".to_string()
        }
        _ => format!("-unknown command '{}'\r\n", first_argument)
    }
}

fn parse_input(input: &str) -> Vec<&str> {
    let mut result = vec![];

    input.split("\r\n")
        .skip(1)
        .enumerate()
        .for_each(|(i, x)| {
            if i % 2 == 1 {
                result.push(x);
            }
        });

    return result;
}
#[cfg(test)]
mod tests {
    use std::collections::HashMap;
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
        let result = redis("*1\r\n$4\r\nPING", &mut HashMap::new());
        assert_eq!(result, "+PONG\r\n");
    }

    #[test]
    fn set() {
        let data = &mut HashMap::new();
        let result = redis("*3\r\n$3\r\nSET\r\n$4\r\nName\r\n$4\r\nJohn\r\n", data);
        assert_eq!(result, "+OK\r\n");
        assert_eq!(data["Name"], "John");
    }

    #[test]
    fn echo() {
        let result = redis("*2\r\n$4\r\nECHO\r\n$11\r\nHello World", &mut HashMap::new());
        assert_eq!(result, "+Hello World\r\n");
    }

    #[test]
    fn echo_missing_arguments() {
        let result = redis("*1\r\n$4\r\nECHO\r\n", &mut HashMap::new());
        assert_eq!(result, "-ERR wrong number of arguments for command\r\n");
    }

    #[test]
    fn echo_too_many_arguments() {
        let result = redis("*3\r\n$4\r\nECHO\r\n$1\r\nN\r\n$1\r\nB\r\n", &mut HashMap::new());
        assert_eq!(result, "-ERR wrong number of arguments for command\r\n");
    }

    #[test]
    fn unknown_command() {
        let result = redis("*1\r\n$4\r\nCIAO\r\n", &mut HashMap::new());
        assert_eq!(result, "-unknown command 'CIAO'\r\n");
    }
}
