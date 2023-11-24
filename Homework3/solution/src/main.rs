use crate::exercise::{Exercise, ExerciseList};
use crate::run::run;
use crate::verify::verify;
use argh::FromArgs;
use console::Emoji;
use notify::DebouncedEvent;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::ffi::OsStr;
use std::fs;
use std::io::{self, prelude::*};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[macro_use]
mod ui;

mod exercise;
mod run;
mod verify;

// In sync with crate version
const VERSION: &str = "4.7.0";

#[derive(FromArgs, PartialEq, Debug)]
/// Zustlings is a collection of small exercises to prepare you for Zero Knowledge Rust 
struct Args {
    /// show outputs from the test exercises
    #[argh(switch)]
    nocapture: bool,
    /// show the executable version
    #[argh(switch, short = 'v')]
    version: bool,
    #[argh(subcommand)]
    nested: Option<Subcommands>,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum Subcommands {
    Verify(VerifyArgs),    
    Run(RunArgs),
    Hint(HintArgs),    
    Homework(HomeworkArgs),
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "verify")]
/// Verifies all exercises according to the recommended order
struct VerifyArgs {}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "run")]
/// Runs/Tests a single exercise
struct RunArgs {
    #[argh(positional)]
    /// the name of the exercise
    name: String,
}


#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "homework")]
/// Runs/Tests a single exercise
struct HomeworkArgs {
    #[argh(positional)]
    /// the day of the homework
    name: String,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "hint")]
/// Returns a hint for the given exercise
struct HintArgs {
    #[argh(positional)]
    /// the name of the exercise
    name: String,
}

fn main() {
    
    let args: Args = argh::from_env();    

    if args.version {
        println!("v{}", VERSION);
        std::process::exit(0);
    }    



    if !Path::new("info.toml").exists() {
        println!(
            "{} must be run from the zustlings directory",
            std::env::current_exe().unwrap().to_str().unwrap()
        );
        println!("Try `cd zustlings/`!");
        std::process::exit(1);
    }

    if !rustc_exists() {
        println!("We cannot find `rustc`.");
        println!("Try running `rustc --version` to diagnose your problem.");
        println!("For instructions on how to install Rust, check the README.");
        std::process::exit(1);
    }

    // Gets homework structs
    let toml_str = &fs::read_to_string("info.toml").unwrap();
    let exercises = toml::from_str::<ExerciseList>(toml_str).unwrap().exercises;
    let verbose = args.nocapture;

    let command = args.nested.unwrap_or_else(|| {
        println!("{}\n", DEFAULT_OUT);
        std::process::exit(0);
    });

    println!("\n\nEND\n\n");    

    match command {  

        Subcommands::Run(subargs) => {
            let exercise = find_exercise(&subargs.name, &exercises);
            run(exercise, verbose).unwrap_or_else(|_| std::process::exit(1));
        }

        Subcommands::Hint(subargs) => {
            let exercise = find_exercise(&subargs.name, &exercises);
            println!("{}", exercise.hint);
        }

        Subcommands::Verify(_subargs) => {
            verify(&exercises, verbose).unwrap_or_else(|_| std::process::exit(1));
        }
        
        Subcommands::Homework(subargs) => match homework(&exercises, verbose, subargs.name) {
            Err(e) => {
                println!("Error: Could not watch your progress. Error message was {:?}.", e);
                println!("Most likely you've run out of disk space or your 'inotify limit' has been reached.");
                std::process::exit(1);
            }
            Ok(WatchStatus::Finished) => {
                println!("{emoji} All exercises completed! {emoji}", emoji = Emoji("🎉", "★"));
                println!("\n{}\n", FENISH_LINE);
            }
            Ok(WatchStatus::Unfinished) => {
                println!("We hope you're enjoying learning about Rust!");
                println!("If you want to continue working on the exercises at a later point, you can simply run `zustlings watch` again");
            }
        },   
    }
}

fn spawn_watch_shell(failed_exercise_hint: &Arc<Mutex<Option<String>>>, should_quit: Arc<AtomicBool>) {
    let failed_exercise_hint = Arc::clone(failed_exercise_hint);
    println!("Welcome to watch mode! You can type 'help' to get an overview of the commands you can use here.");
    thread::spawn(move || loop {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let input = input.trim();
                if input == "hint" {
                    if let Some(hint) = &*failed_exercise_hint.lock().unwrap() {
                        println!("{}", hint);
                    }
                } else if input == "clear" {
                    println!("\x1B[2J\x1B[1;1H");
                } else if input.eq("quit") {
                    should_quit.store(true, Ordering::SeqCst);
                    println!("Bye!");
                } else if input.eq("help") {
                    println!("Commands available to you in watch mode:");
                    println!("  hint  - prints the current exercise's hint");
                    println!("  clear - clears the screen");
                    println!("  quit  - quits watch mode");
                    println!("  help  - displays this help message");
                    println!();
                    println!("Watch mode automatically re-evaluates the current exercise");
                    println!("when you edit a file's contents.")
                } else {
                    println!("unknown command: {}", input);
                }
            }
            Err(error) => println!("error reading command: {}", error),
        }
    });
}

fn find_exercise<'a>(name: &str, exercises: &'a [Exercise]) -> &'a Exercise {
    if name.eq("next") {
        exercises.iter().find(|e| !e.looks_done()).unwrap_or_else(|| {
            println!("🎉 Congratulations! You have done all the exercises!");
            println!("🔚 There are no more exercises to do next!");
            std::process::exit(1)
        })
    } else {
        exercises.iter().find(|e| e.name == name).unwrap_or_else(|| {
            println!("No exercise found for '{}'!", name);
            std::process::exit(1)
        })
    }
}

enum WatchStatus {
    Finished,
    Unfinished,
}

// Redo watch but with only the homework that has been given out
// pass on the way here the subset
fn homework(exercises: &[Exercise], verbose: bool, homework_number: String) -> notify::Result<WatchStatus> {
   
     /* Clears the terminal with an ANSI escape code.
    Works in UNIX and newer Windows terminals. */
    fn clear_screen() {
        println!("\x1Bc");
    }

    println!("exercises: {:?}", exercises);
    println!("exercise[0]: {:?}", exercises.len());    

    let (tx, rx) = channel();
    let should_quit = Arc::new(AtomicBool::new(false));

    // load from the other dir 
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(2))?;

    // watch for changes anywhere in this folder (TODO check subdir check)    
    watcher.watch(Path::new("./homeworks"), RecursiveMode::Recursive)?;  // I think watcher looks for file changes
    clear_screen(); 

    let mut homework_path: String = "./homeworks/homework".to_owned();        
    homework_path.push_str(&homework_number);
    
    let to_owned_hint = |t: &Exercise| t.hint.to_owned();

    // Check if directory exists
    let b = Path::new(&homework_path).is_dir();   //.is_file();
    
    // Filter against what's in the dirctory for number 1
    let paths = fs::read_dir(homework_path).expect("Can't find homework. Have you run the wrong homework number?");

    // Vec of things that are present in this homework
    let mut exercise_names: Vec<String> = Vec::new();

    for path in paths {            
        let strr: String = path.unwrap().path().display().to_string();       
        let res: Vec<String> = strr.split("/").map(|s| s.to_string()).collect();     
        exercise_names.push(res.last().unwrap().clone());
    }
    
    let mut exercises_filtered: Vec<exercise::Exercise> = Vec::new();

    println!("\n");

    // filter out from the exercises list.
    // Include based on matching homework subdirectories
    for exercise in exercises {            
        
        let path_string: String = exercise.path.clone().into_os_string().into_string().unwrap();        
        let path_elements: Vec<&str> = path_string.split('/').collect();       

        if path_elements.len() > 3{

            let exercise_dir: String = String::from(path_elements.clone()[2]);           

            // retrieve 3rd variable from teh path and check for match
            if exercise_names.contains(&exercise_dir) {
                exercises_filtered.push(exercise.clone());
            }
        }
    }    
    
    // println!("\nExercises in this homework set:");
    // for exercise in exercises_filtered.clone() {           
    //     println!("exercises_filtered: {:?}",exercise.name);
    // }    

    // pass here for looping till done
    let failed_exercise_hint = match verify(exercises_filtered.iter(), verbose) {
        Ok(_) => return Ok(WatchStatus::Finished),
        Err(exercise) => Arc::new(Mutex::new(Some(to_owned_hint(exercise)))),
    };    

    println!("Spawning homeworkd watch shell");
    spawn_watch_shell(&failed_exercise_hint, Arc::clone(&should_quit));
    loop {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(event) => match event {
                DebouncedEvent::Create(b) | DebouncedEvent::Chmod(b) | DebouncedEvent::Write(b) => {
                    if b.extension() == Some(OsStr::new("rs")) && b.exists() {
                        let filepath = b.as_path().canonicalize().unwrap();
                        let pending_exercises = exercises_filtered
                            .iter()
                            .skip_while(|e| !filepath.ends_with(&e.path))
                            // .filter(|e| filepath.ends_with(&e.path))
                            .chain(exercises_filtered.iter().filter(|e| !e.looks_done() && !filepath.ends_with(&e.path)));
                        clear_screen();

                        match verify(pending_exercises, verbose) {
                            Ok(_) => return Ok(WatchStatus::Finished),
                            Err(exercise) => {
                                let mut failed_exercise_hint = failed_exercise_hint.lock().unwrap();
                                *failed_exercise_hint = Some(to_owned_hint(exercise));
                            }
                        }
                    }
                }
                _ => {}
            },
            Err(RecvTimeoutError::Timeout) => {
                // the timeout expired, just check the `should_quit` variable below then loop again
            }
            Err(e) => println!("watch error: {:?}", e),
        }
        // Check if we need to exit
        if should_quit.load(Ordering::SeqCst) {
            return Ok(WatchStatus::Unfinished);
        }
    }
}


fn rustc_exists() -> bool {
    Command::new("rustc")
        .args(&["--version"])
        .stdout(Stdio::null())
        .spawn()
        .and_then(|mut child| child.wait())
        .map(|status| status.success())
        .unwrap_or(false)
}

const DEFAULT_OUT: &str = r#"Thanks for installing Zustlings!

How Zustlings works

1. The central concept behind Zustlings is that you solve exercises. These
   exercises usually have some sort of syntax error in them, which will cause
   them to fail compilation or testing. Sometimes there's a logic error instead
   of a syntax error. No matter what error, it's your job to find it and fix it!
   You'll know when you fixed it because then, the exercise will compile and
   Zustlings will be able to move on to the next exercise.
2. If you run Zustlings in watch mode (which we recommend), it'll automatically
   start with the first exercise. Don't get confused by an error message popping
   up as soon as you run Zustlings! This is part of the exercise that you're
   supposed to solve, so open the exercise file in an editor and start your
   detective work!
3. If you're stuck on an exercise, there is a helpful hint you can view by typing
   'hint' (in watch mode), or running `zustlings hint exercise_name`.

Got all that? Great! To get started, run `zustlings watch` in order to get the first
exercise. Make sure to have your editor open!"#;

const FENISH_LINE: &str = r#"+----------------------------------------------------+
|          You made it to the End of this Homework!          |
+--------------------------  ------------------------+
                          \\/
     ▒▒          ▒▒▒▒▒▒▒▒      ▒▒▒▒▒▒▒▒          ▒▒
   ▒▒▒▒  ▒▒    ▒▒        ▒▒  ▒▒        ▒▒    ▒▒  ▒▒▒▒
   ▒▒▒▒  ▒▒  ▒▒            ▒▒            ▒▒  ▒▒  ▒▒▒▒
 ░░▒▒▒▒░░▒▒  ▒▒            ▒▒            ▒▒  ▒▒░░▒▒▒▒
   ▓▓▓▓▓▓▓▓  ▓▓      ▓▓██  ▓▓  ▓▓██      ▓▓  ▓▓▓▓▓▓▓▓
     ▒▒▒▒    ▒▒      ████  ▒▒  ████      ▒▒░░  ▒▒▒▒
       ▒▒  ▒▒▒▒▒▒        ▒▒▒▒▒▒        ▒▒▒▒▒▒  ▒▒
         ▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▒▒▒▒▒▒▒▒▓▓▒▒▓▓▒▒▒▒▒▒▒▒
           ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒
             ▒▒▒▒▒▒▒▒▒▒██▒▒▒▒▒▒██▒▒▒▒▒▒▒▒▒▒
           ▒▒  ▒▒▒▒▒▒▒▒▒▒██████▒▒▒▒▒▒▒▒▒▒  ▒▒
         ▒▒    ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒    ▒▒
       ▒▒    ▒▒    ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒    ▒▒    ▒▒
       ▒▒  ▒▒    ▒▒                  ▒▒    ▒▒  ▒▒
           ▒▒  ▒▒                      ▒▒  ▒▒

We hope you enjoyed learning about the various aspects of Rust!
If you noticed any issues, please don't hesitate to report them to our repo.
You can also contribute your own exercises to help the greater community!



const WELCOME: &str = r#"       welcome to... Zustlins"#;
