use std::{env::{self, Args}, fmt::Display, rc::Rc};
use anyhow::anyhow;
use uuid::Uuid;

struct Arg {
    long: &'static str,
    short: char,
}

impl Display for Arg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.long, self.short)    
    }
}

struct Cli<T: ?Sized> {
    possible_args: Vec<Rc<T>>,
    pub final_args: Vec<Rc<T>>,
}

impl<T> Cli<T> where T: TArg {
    fn new(a: Vec<Rc<T>>) -> Self {
        Self {
            possible_args: a,
            final_args: Default::default(),
        }
    }
}

impl<T> Cli<T> where T: TArg {
    fn from(incoming: Args, possibilities: Vec<Rc<T>>) -> Self {
        let mut cli = Self::new(possibilities);
        let mut incoming = incoming.enumerate();

        loop {
            if let Some((index, word)) = incoming.next() {
                // see if this arg matches any valid args
                if word.starts_with("--") {
                    // long arg
                    // remove the -- prefix
                    if let Some((_, word)) = word.split_at_checked(2) {
                        // find what argument this is supposed to be
                        if let Some(value) = cli.possible_args.iter_mut().find(|a| a.get_names().long == word) {
                            if value.takes_arg() {
                                // advances the iterator, that's why we check first
                                if let Some((_, arg)) = incoming.next() {
                                    value.set_arg(&arg);
                                }
                            }
                            cli.final_args.push(value.clone());
                        }
                    } else {
                        // blank --
                        // used for stdout?
                        panic!("Argument {index}; \"--\" used with no string after it");
                    }
                } else if word.starts_with("-") {
                    // short arg 

                    // remove - prefix
                    if let Some((_, word)) = word.split_at_checked(1) {
                        // extract all the arguments from the chars
                        word
                            .chars()
                            .into_iter()
                            .for_each(|c| {
                                cli.possible_args.iter_mut()
                                // find all args that match the char
                                .filter(|v| v.get_names().short == c)
                                // see what arguments need the next value
                                .fold(false, |last, f| {
                                    // only one argument can request the following value
                                    if f.takes_arg() && last { panic!("Argument {index}; Multiple shorthand arguments require the next value, too ambagious!"); }
                                    // give this argument the following value
                                    if f.takes_arg() {
                                        match incoming.next() {
                                            Some((_,value)) => {
                                                f.set_arg(&value);
                                                cli.final_args.push(f.clone())
                                            },
                                            None => panic!("Argument {index}; {} wanted a value to be passed, but no value was.", f.get_names()),
                                        }
                                    }
                                    f.takes_arg()
                                });
                            });
                    } else {
                        panic!("Argument {index}; '-' used with no chars after it.");
                    }
                } else {
                    // value
                    panic!("Argument {index}; Unexpected value {} (previous argument didn't request a value to be passed).", word);
                }
            } else {
                // end of arguments
                break;
            }
        }
        // cli
        todo!()
    }
}

trait TArg {
    fn takes_arg(&self) -> bool;
    fn get_names(&self) -> Arg;
    fn set_arg(&mut self, arg: &str);
}

pub async fn parse() -> Result<(), anyhow::Error> {
    
    let args: Cli<dyn TArg> = Cli::from(env::args(), vec![Rc::new(Gen::default()), Rc::new(UsbDevice::default())]);


    Err(anyhow!(HELPTEXT))
}

fn generate(next: &str) {
    let id = Uuid::new_v4();
    println!("{id},{next}");

} 

const HELPTEXT: &str = r#"
There is no default behavior.

--generate NAME     Generate a new id/location pair and write it to standard out in CSV format.
                    (Strings with spaces must be wrapped in quotes.)
--serve             Start the server
"#;

#[derive(Default, Clone)]
struct Gen {
    location_name: String,
}

impl TArg for Gen {
    fn takes_arg(&self) -> bool {
        true
    }

    fn get_names(&self) -> Arg {
        Arg { long: "generate", short: 'g' }
    }
    
    fn set_arg(&mut self, arg: &str) {
        self.location_name = arg.to_string();
    } 
}

#[derive(Default, Clone)]
struct UsbDevice {
    path: String,
}

impl TArg for UsbDevice {
    fn takes_arg(&self) -> bool {
        true
    }

    fn get_names(&self) -> Arg {
        Arg { long: "dev", short: 'd' }
    }

    fn set_arg(&mut self, arg: &str) {
        self.path = arg.to_string();
    }
}



