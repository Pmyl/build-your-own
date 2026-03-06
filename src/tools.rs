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
            fn from_str(s: &str) -> Option<Self> {
                match s {
                    $(
                        $command => Some(Tool::$variant),
                    )+
                    _ => None,
                }
            }

            fn list() {
                println!("Tools:");
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
            if let Some(tool_name) = args.get(0).map(|s| *s) {
                let tool = Tool::from_str(tool_name);
                match tool {
                    Some(tool) => match (match tool {
                        $(
                            Tool::$variant => $function(&args.iter().skip(1).map(|s| &**s).collect::<Vec<&str>>()),
                        )+
                    }) {
                        Err(e) => eprintln!("{}", e),
                        Ok(_) => (),
                    },
                    _ => {
                        eprintln!("!!!!!!!!!!{}!!!!!!!!!!!!!!!!!!!!", "!".repeat(tool_name.len()));
                        eprintln!("!!! Tool [{}] not configured !!!", tool_name);
                        eprintln!("!!!!!!!!!!{}!!!!!!!!!!!!!!!!!!!!\n", "!".repeat(tool_name.len()));
                        Tool::list()
                    },
                }
            } else {
                Tool::list()
            };
        }
    };
}
