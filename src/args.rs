use std::borrow::Borrow;
use std::fmt::{Debug, Formatter};
use std::process::exit;

use crate::report::{Level, ReportKind};

macro_rules! error {
    ($($ident:tt)*) => {
        ReportKind::ArgumentParserError
            .title(format!($($ident)*))
            .note("(Run with \x1b[1m--help\x1b[0m for usage information)");
        exit(1);
    };
}

#[derive(Copy, Clone)]
pub struct Arg<T> {
    pub value: T,
    set:       bool,
}

impl<T> Arg<T> {
    fn new(default: T) -> Self {
        Self { value: default, set: false }
    }

    fn try_mut<N: std::fmt::Display>(&mut self, name: N, value: T) {
        if self.set {
            error!("'{}' may only be used once", name);
        }
        self.value = value;
    }
}

impl<T> std::ops::Deref for Arg<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: Debug> Debug for Arg<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.value)
    }
}

#[derive(Debug)]
pub struct Args {
    pub file:         Arg<&'static str>,
    pub output:       Arg<&'static str>,
    pub debug:        Arg<bool>,
    pub code_context: Arg<bool>,
    pub level:        Arg<Level>,
    pub verbs:        Vec<&'static str>,
}

impl Args {
    pub fn default() -> Self {
        Self {
            file:         Arg::new("main.shd"),
            output:       Arg::new("main.asm"),
            debug:        Arg::new(false),
            code_context: Arg::new(true),
            level:        Arg::new(Level::Warn),
            verbs:        Vec::new(),
        }
    }

    fn handle_arg(&mut self, argument: &str, arguments: &mut std::vec::IntoIter<String>) {
        let args: Vec<String> = if argument.starts_with("--") {
            vec![argument.into()]
        }
        else {
            argument.chars().skip(1).map(|c| format!("-{c}")).collect()
        };
        let args_len = args.len();

        for (i, arg) in args.into_iter().enumerate() {
            let is_end = i == args_len - 1;

            macro_rules! is_end {
                () => {
                    if !is_end {
                        error!("{} may only be used at the end of a group", arg);
                    }
                };
            }
            match arg.as_str() {
                "-h" => {
                    println!("{USAGE}");
                    exit(0);
                },
                "--help" => {
                    println!("{USAGE}\n\n{HELP_MESSAGE}");
                    exit(0);
                },
                "-V" | "--version" => {
                    println!("sharc {}", env!("CARGO_PKG_VERSION"));
                    exit(0);
                },
                "-d" | "--debug" => self.debug.try_mut(arg, true),
                "-f" | "--file" => {
                    is_end!();
                    let file = arguments.next().unwrap_or_else(|| {
                        error!("{arg} expected FILE");
                    });

                    self.file.try_mut(arg, Box::leak(file.into_boxed_str()));
                },
                "-o" | "--output" => {
                    is_end!();
                    let output = arguments.next().unwrap_or_else(|| {
                        error!("expected file");
                    });

                    self.output.try_mut(arg, Box::leak(output.into_boxed_str()));
                },
                "-l" | "--error-level" => {
                    is_end!();
                    let level = arguments.next().unwrap_or_else(|| {
                        error!("expected level");
                    });

                    self.level.try_mut(arg, match level.as_str() {
                        "f" | "fatal" => Level::Fatal,
                        "e" | "error" => Level::Error,
                        "w" | "warn" => Level::Warn,
                        "n" | "note" => Level::Note,
                        "s" | "silent" => Level::Silent,
                        _ => {
                            error!("invalid level `{level}`");
                        },
                    });
                },
                "--no-context" => self.code_context.try_mut(arg, false),

                _ => {
                    error!("unrecognized argument {arg}");
                },
            }
        }
    }

    pub fn parse(args: Vec<String>) -> Self {
        let mut out = Self::default();
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            if arg.starts_with('-') {
                out.handle_arg(&arg, &mut args);
                continue;
            }

            if arg == "shark" {
                println!("\x1b[34m{SHARK_ASCII}\x1b[0m");
                exit(1);
            }

            out.verbs.push(Box::leak(arg.into_boxed_str()));
        }

        // drain remaining args
        for arg in args.by_ref() {
            out.verbs.push(Box::leak(arg.into_boxed_str()));
        }

        out
    }
}

const USAGE: &str = "Usage: sharc [-hVd] [-l LEVEL] [-f FILE] [-o FILE] [VERB...]";
const HELP_MESSAGE: &str = "\x1b[1mDESCRIPTION\x1b[0m
    The compiler for the Shard Programming Language.
    Documentation can be found at https://shardlang.org/doc/

\x1b[1mOPTIONS\x1b[0m
    -h, --help                  Show only usage with -h
    -v, --version               Show version
    -d, --debug                 Print debug information
        Shows a ton of information not intended for mere mortals.
    -l, --error-level LEVEL     [fatal|error|warn|note|silent]
        (default: warn)
    -f, --file FILE             File to compile
        (default: main.shd)
    -o, --output FILE           File to write to
        (default: main.asm)

        --no-context            Disable code context";
const SHARK_ASCII: &str = r#"                                 ,-
                               ,'::|
                              /::::|
                            ,'::::o\                                      _..
         ____........-------,..::?88b                                  ,-' /
 _.--"""". . . .      .   .  .  .  ""`-._                           ,-' .;'
<. - :::::o......  ...   . . .. . .  .  .""--._                  ,-'. .;'
 `-._  ` `":`:`:`::||||:::::::::::::::::.:. .  ""--._ ,'|     ,-'.  .;'
     """_=--       //'doo.. ````:`:`::::::::::.:.:.:. .`-`._-'.   .;'
         ""--.__     P(       \               ` ``:`:``:::: .   .;'
                "\""--.:-.     `.                             .:/
                  \. /    `-._   `.""-----.,-..::(--"".\""`.  `:\
                   `P         `-._ \          `-:\          `. `:\
                                   ""            "            `-._)"#;
