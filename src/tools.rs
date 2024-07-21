#[macro_export]
macro_rules! tools {
    (
        enum Tool {
            $(
                #[tool(command = $command:expr, description = $description:expr, function = $function:path)]
                $variant:ident,
            )+
        }
    ) => {
        #[derive(Debug)]
        enum Tool {
            $(
                $variant,
            )+
        }

        impl Tool {
            fn from_str(s: Option<&str>) -> Option<Self> {
                match s {
                    $(
                        Some($command) => Some(Tool::$variant),
                    )+
                    Some(other) => panic!("Tool [{}] not configured", other),
                    _ => None,
                }
            }

            fn list() {
                println!("Tool <usage> - Example:");
                $(
                    println!("{}", $description);
                )+
            }
        }

        fn main() {
            let args: Vec<String> = env::args().skip(1).collect();

            // args structure is:
            // 0: executable name (not interested)
            // 1..n: parameters
            let args: Vec<&str> = args.iter().map(|s| &**s).collect::<Vec<&str>>();

            // parameters structure has to be:
            // 0: tool name
            // 1..n: tool parameters
            let tool = Tool::from_str(args.get(0).map(|s| *s));

            match tool {
                Some(tool) => match (match tool {
                    $(
                        Tool::$variant => $function(&args.iter().skip(1).map(|s| &**s).collect::<Vec<&str>>()),
                    )+
                }) {
                    Err(MyOwnError::ActualError(e)) => panic!("{}", e),
                    Err(MyOwnError::ActualErrorWithDescription(e, description)) => panic!("{}: {}", description, e),
                    _ => (),
                },
                None => Tool::list(),
            }
        }
    };
}
