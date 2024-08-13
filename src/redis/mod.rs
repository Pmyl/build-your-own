use build_your_own_macros::cli_options;
use build_your_own_utils::my_own_error::MyOwnError;
use std::collections::HashMap;
use std::str::FromStr;
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

    let mut redis = Redis::default();
    let redis_time_provider: Box<dyn TimeProvider> = Box::new(RedisTimeProvider);

    for stream in listener.incoming() {
        let mut stream = stream?;

        loop {
            let mut buffer = [0; 1024];
            let bytes_read = stream.read(&mut buffer)?;

            // Check if the client closed the connection
            if bytes_read == 0 {
                break;
            }

            let request = String::from_utf8_lossy(&buffer);

            let response = redis.process(&request, &redis_time_provider);
            stream.write(response.as_bytes())?;
        }
    }

    Ok(())
}

struct RedisTimeProvider;

impl TimeProvider for RedisTimeProvider {
    fn now(&self) -> Instant {
        Instant::now()
    }
}

cli_options! {
    struct RedisConfig {
        #[option(name = "-p", default = 6379)]
        port: u16
    }
}

struct Redis {
    data: HashMap<String, (String, Option<Instant>)>,
}

impl Redis {
    fn default() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    fn process(&mut self, input: &str, time_provider: &Box<dyn TimeProvider>) -> String {
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
                if arguments.len() > 3 {
                    let expire = arguments[4];
                    match u64::from_str(expire) {
                        Ok(expire) => {
                            self.data
                                .insert(arguments[1].to_string(), (arguments[2].to_string(), Some(time_provider.now() + Duration::from_secs(expire))));
                        }
                        Err(_) => {
                            todo!()
                        }
                    }
                } else {
                    self.data
                        .insert(arguments[1].to_string(), (arguments[2].to_string(), None));
                }
                "+OK\r\n".to_string()
            }
            "GET" => {
                let value = self.data.get(&arguments[1].to_string());

                match value {
                    None => "$-1\r\n".to_string(),
                    Some(value) => {
                        if value.1.is_some() {
                            if value.1.unwrap() > time_provider.now() {
                                return format!("+{}\r\n", value.0);
                            } else {
                                return "$-1\r\n".to_string();
                            }
                        }
                        format!("+{}\r\n", value.0)
                    }
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
        let mut redis = Redis::default();

        let result = redis.process("*1\r\n$4\r\nPING", &FakeTimeProvider::new_now());
        assert_eq!(result, "+PONG\r\n");
    }

    #[test]
    fn set() {
        let mut redis = Redis::default();
        let result = redis.process("*3\r\n$3\r\nSET\r\n$4\r\nName\r\n$4\r\nJohn\r\n", &FakeTimeProvider::new_now());
        assert_eq!(result, "+OK\r\n");

        let result = redis.process("*2\r\n$3\r\nGET\r\n$4\r\nName\r\n", &FakeTimeProvider::new_now());

        assert_eq!(result, "+John\r\n");
    }

    struct FakeTimeProvider {
        now: Instant,
    }

    impl FakeTimeProvider {}

    impl FakeTimeProvider {
        fn new_now() -> Box<dyn TimeProvider> {
            Box::new(FakeTimeProvider { now: Instant::now() })
        }

        fn new(instant: Instant) -> Self {
            FakeTimeProvider { now: instant }
        }
    }

    impl TimeProvider for FakeTimeProvider {
        fn now(&self) -> Instant {
            self.now
        }
    }

    #[test]
    fn set_expire() {
        let mut redis = Redis::default();
        let instant = Instant::now();
        let result = redis.process("*5\r\n$3\r\nSET\r\n$4\r\nName\r\n$4\r\nJohn\r\n$2\r\nEX\r\n$2\r\n60\r\n", &(Box::new(FakeTimeProvider::new(instant)) as Box<dyn TimeProvider>));
        assert_eq!(result, "+OK\r\n");

        let result = redis.process("*2\r\n$3\r\nGET\r\n$4\r\nName\r\n", &(Box::new(FakeTimeProvider::new(instant + Duration::from_secs(59))) as Box<dyn TimeProvider>));
        assert_eq!(result, "+John\r\n");

        let result = redis.process("*2\r\n$3\r\nGET\r\n$4\r\nName\r\n", &(Box::new(FakeTimeProvider::new(instant + Duration::from_secs(60))) as Box<dyn TimeProvider>));
        assert_eq!(result, "$-1\r\n");
    }

    #[test]
    fn get_missing() {
        let mut redis = Redis::default();

        let result = redis.process("*2\r\n$3\r\nGET\r\n$4\r\nName\r\n", &FakeTimeProvider::new_now());

        assert_eq!(result, "$-1\r\n");
    }

    #[test]
    fn echo() {
        let mut redis = Redis::default();
        let result = redis.process("*2\r\n$4\r\nECHO\r\n$11\r\nHello World", &FakeTimeProvider::new_now());
        assert_eq!(result, "+Hello World\r\n");
    }

    #[test]
    fn echo_missing_arguments() {
        let mut redis = Redis::default();
        let result = redis.process("*1\r\n$4\r\nECHO\r\n", &FakeTimeProvider::new_now());
        assert_eq!(result, "-ERR wrong number of arguments for command\r\n");
    }

    #[test]
    fn echo_too_many_arguments() {
        let mut redis = Redis::default();
        let result = redis.process("*3\r\n$4\r\nECHO\r\n$1\r\nN\r\n$1\r\nB\r\n", &FakeTimeProvider::new_now());
        assert_eq!(result, "-ERR wrong number of arguments for command\r\n");
    }

    #[test]
    fn unknown_command() {
        let mut redis = Redis::default();
        let result = redis.process("*1\r\n$4\r\nCIAO\r\n", &FakeTimeProvider::new_now());
        assert_eq!(result, "-unknown command 'CIAO'\r\n");
    }
}
