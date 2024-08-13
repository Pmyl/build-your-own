use build_your_own_macros::cli_options;
use build_your_own_utils::my_own_error::MyOwnError;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
use std::{
    io::{Read, Write},
    net::TcpListener,
};
// https://codingchallenges.fyi/challenges/challenge-redis

pub fn redis_cli(args: &[&str]) -> Result<(), MyOwnError> {
    let redis_config = RedisConfig::from_args(args)?;
    let listener = TcpListener::bind(format!("127.0.0.1:{}", redis_config.port))?;
    println!("Listening on port {}", redis_config.port);

    let redis = Redis::default();

    thread::scope(|scope| {
        for stream in listener.incoming() {
            scope.spawn(|| {
                let mut stream = stream.expect("Expect stream to be valid");

                loop {
                    let mut buffer = [0; 1024];
                    let bytes_read = stream
                        .read(&mut buffer)
                        .expect("Failed to read from stream");

                    if bytes_read == 0 {
                        break;
                    }

                    let request = String::from_utf8_lossy(&buffer);

                    redis
                        .process(&request, &mut stream, &Instant::now())
                        .expect("Failed to process request");
                }
            });
        }
    });

    Ok(())
}

impl TimeProvider for Instant {
    fn now(&self) -> Instant {
        *self
    }
}

cli_options! {
    struct RedisConfig {
        #[option(name = "-p", default = 6379)]
        port: u16
    }
}

struct Redis {
    data: Mutex<HashMap<String, (String, Option<Instant>)>>,
}

impl Redis {
    fn default() -> Self {
        Self {
            data: Mutex::new(HashMap::new()),
        }
    }

    fn process(
        &self,
        input: &str,
        mut output: impl Write,
        time_provider: &impl TimeProvider,
    ) -> Result<(), MyOwnError> {
        let arguments = parse_input(input);

        let first_argument = arguments[0];

        let response: &[u8] = match first_argument {
            "ECHO" => {
                if arguments.len() != 2 {
                    b"-ERR wrong number of arguments for command\r\n"
                } else {
                    &format!("+{}\r\n", arguments[1]).into_bytes()
                }
            }
            "PING" => b"+PONG\r\n",
            "SET" => {
                if arguments.len() > 3 {
                    let expire = arguments[4];
                    match u64::from_str(expire) {
                        Ok(expire) => {
                            self.data.lock().unwrap().insert(
                                arguments[1].to_string(),
                                (
                                    arguments[2].to_string(),
                                    Some(time_provider.now() + Duration::from_secs(expire)),
                                ),
                            );
                        }
                        Err(_) => {
                            todo!()
                        }
                    }
                } else {
                    self.data
                        .lock()
                        .unwrap()
                        .insert(arguments[1].to_string(), (arguments[2].to_string(), None));
                }

                b"+OK\r\n"
            }
            "GET" => {
                let hash_map = self.data.lock().unwrap();
                let value = hash_map.get(&arguments[1].to_string());

                match value {
                    Some((value, None)) => &format!("+{}\r\n", value).into_bytes(),
                    Some((value, Some(exp))) if exp > &time_provider.now() => {
                        &format!("+{}\r\n", value).into_bytes()
                    }
                    _ => b"$-1\r\n",
                }
            }
            _ => &format!("-unknown command '{}'\r\n", first_argument).into_bytes(),
        };

        output.write_all(response)?;

        Ok(())
    }
}

fn parse_input(input: &str) -> Vec<&str> {
    let mut result = vec![];

    input.split("\r\n").skip(1).enumerate().for_each(|(i, x)| {
        if i % 2 == 1 {
            result.push(x);
        }
    });

    result
}

trait TimeProvider {
    fn now(&self) -> Instant;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn parse_input_test() {
        let result = parse_input("*1\r\n$4\r\nPING\r\n");
        assert_eq!(result, vec!["PING"]);

        let result = parse_input("*2\r\n$4\r\nECHO\r\n$11\r\nHello World");
        assert_eq!(result, vec!["ECHO", "Hello World"]);
    }

    #[test]
    fn pong() {
        let redis = Redis::default();
        let mut output = Vec::new();

        redis
            .process("*1\r\n$4\r\nPING", &mut output, &Instant::now())
            .expect("Failed to process");
        assert_eq!(output, b"+PONG\r\n");
    }

    #[test]
    fn set() {
        let redis = Redis::default();
        let mut output = Vec::new();

        redis
            .process(
                "*3\r\n$3\r\nSET\r\n$4\r\nName\r\n$4\r\nJohn\r\n",
                &mut output,
                &Instant::now(),
            )
            .expect("Failed to process");
        assert_eq!(output, b"+OK\r\n");

        let mut output = Vec::new();
        redis
            .process(
                "*2\r\n$3\r\nGET\r\n$4\r\nName\r\n",
                &mut output,
                &Instant::now(),
            )
            .expect("Failed to process");
        assert_eq!(output, b"+John\r\n");
    }

    #[test]
    fn set_expire() {
        let redis = Redis::default();
        let mut output = Vec::new();
        let instant = Instant::now();

        redis
            .process(
                "*5\r\n$3\r\nSET\r\n$4\r\nName\r\n$4\r\nJohn\r\n$2\r\nEX\r\n$2\r\n60\r\n",
                &mut output,
                &instant,
            )
            .expect("Failed to process");
        assert_eq!(output, b"+OK\r\n");

        let mut output = Vec::new();
        redis
            .process(
                "*2\r\n$3\r\nGET\r\n$4\r\nName\r\n",
                &mut output,
                &(instant + Duration::from_secs(59)),
            )
            .expect("Failed to process");
        assert_eq!(output, b"+John\r\n");

        let mut output = Vec::new();
        redis
            .process(
                "*2\r\n$3\r\nGET\r\n$4\r\nName\r\n",
                &mut output,
                &(instant + Duration::from_secs(60)),
            )
            .expect("Failed to process");
        assert_eq!(output, b"$-1\r\n");
    }

    #[test]
    fn get_missing() {
        let redis = Redis::default();
        let mut output = Vec::new();

        redis
            .process(
                "*2\r\n$3\r\nGET\r\n$4\r\nName\r\n",
                &mut output,
                &Instant::now(),
            )
            .expect("Failed to process");

        assert_eq!(output, b"$-1\r\n");
    }

    #[test]
    fn echo() {
        let redis = Redis::default();
        let mut output = Vec::new();
        redis
            .process(
                "*2\r\n$4\r\nECHO\r\n$11\r\nHello World",
                &mut output,
                &Instant::now(),
            )
            .expect("Failed to process");
        assert_eq!(output, b"+Hello World\r\n");
    }

    #[test]
    fn echo_missing_arguments() {
        let redis = Redis::default();
        let mut output = Vec::new();
        redis
            .process("*1\r\n$4\r\nECHO\r\n", &mut output, &Instant::now())
            .expect("Failed to process");
        assert_eq!(output, b"-ERR wrong number of arguments for command\r\n");
    }

    #[test]
    fn echo_too_many_arguments() {
        let redis = Redis::default();
        let mut output = Vec::new();
        redis
            .process(
                "*3\r\n$4\r\nECHO\r\n$1\r\nN\r\n$1\r\nB\r\n",
                &mut output,
                &Instant::now(),
            )
            .expect("Failed to process");
        assert_eq!(output, b"-ERR wrong number of arguments for command\r\n");
    }

    #[test]
    fn unknown_command() {
        let redis = Redis::default();
        let mut output = Vec::new();
        redis
            .process("*1\r\n$4\r\nCIAO\r\n", &mut output, &Instant::now())
            .expect("Failed to process");
        assert_eq!(output, b"-unknown command 'CIAO'\r\n");
    }
}
