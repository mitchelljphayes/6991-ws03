use termgame::{
    run_game, CharChunkMap, Controller, Game, GameEvent, GameSettings, KeyCode, SimpleEvent,
};

use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::time::Duration;
// use std::io::Write;
/// This is a single "buffer".
struct Buffer {
    text: String,
}

impl Buffer {
    /// This creates a new Buffer, to use it you should run:
    /// ```rust
    /// Buffer::new()
    /// ```
    fn new() -> Buffer {
        Buffer {
            text: String::new(),
        }
    }

    /// A [`CharChunkMap`] is how termgame stores characters.
    /// This converts a buffer into something which can be shown on screen.
    /// You will likely not need to change this function.
    fn chunkmap_from_textarea(&mut self, map: &mut CharChunkMap) {
        let (mut line, mut col) = (0, 0);
        for c in self.text.chars() {
            map.insert(col, line, c.into());
            col += 1;
            if c == '\n' {
                line += 1;
                col = 0;
            }
        }
    }

    /// Adds a char to the end of the buffer.
    fn push_char(&mut self, c: char) {
        self.text.push(c);
    }

    /// Removes the last char in the buffer.
    fn pop_char(&mut self) {
        self.text.pop();
    }

    // /// This is an example of a function that takes the Buffer as owned,
    // /// as well as another text area; and returns a new Buffer.
    // /// You would either need to return a `Buffer`, or be sure that
    // /// the user will not want the `Buffer` anymore.
    // fn example_owned(self, another_arg: Buffer) -> Buffer {
    //    todo!()
    // }

    // /// This is an example of a function that takes the Buffer by
    // /// mutable reference.
    // fn example_ref_mut(&mut self, another_arg: i32) {
    //     todo!()
    // }

    // /// This is an example of a function that takes the Buffer by
    // /// reference.
    // fn example_ref(&self) -> i32 {
    //     todo!()
    // }
}

/// This struct implements all the
/// logic for how the editor should work. It
/// implements "Controller", which defines how
/// something should interact with the terminal.
struct BufferEditor {
    buffer: Buffer,
}

impl Controller for BufferEditor {
    /// This gets run once, you can probably ignore it.
    fn on_start(&mut self, game: &mut Game) {
        let mut chunkmap = CharChunkMap::new();
        self.buffer.chunkmap_from_textarea(&mut chunkmap);
        game.swap_chunkmap(&mut chunkmap);
    }

    /// Any time there's a keypress, you'll get this
    /// function called.
    fn on_event(&mut self, game: &mut Game, event: GameEvent) {
        match event.into() {
            SimpleEvent::Just(KeyCode::Char(c)) => self.buffer.push_char(c),
            SimpleEvent::Just(KeyCode::Enter) => self.buffer.push_char('\n'),
            SimpleEvent::Just(KeyCode::Backspace) => self.buffer.pop_char(),
            SimpleEvent::Just(KeyCode::Esc) => {
                game.end_game();
            }
            SimpleEvent::Just(KeyCode::Up) => {
                let mut viewport = game.get_viewport();
                if viewport.y > 0 {
                    viewport.y -= 1;
                }
                game.set_viewport(viewport)
            }
            SimpleEvent::Just(KeyCode::Down) => {
                let mut viewport = game.get_viewport();
                viewport.y += 1;
                game.set_viewport(viewport)
            }
            _ => {}
        }
        let mut chunkmap = CharChunkMap::new();
        self.buffer.chunkmap_from_textarea(&mut chunkmap);
        game.swap_chunkmap(&mut chunkmap);
    }

    /// This function gets called regularly, so you can use it
    /// for logic that's independent of key-presses like
    /// implementing a "mouse".
    fn on_tick(&mut self, _game: &mut Game) {}
}

fn run_command(
    cmd: &str,
    buffers: &mut HashMap<String, BufferEditor>,
) -> Result<(), Box<dyn Error>> {
    let input: Vec<&str> = cmd.split_ascii_whitespace().collect();

    fn create_file(filename: &str) -> &str {
        File::create(filename).unwrap();
        std::fs::write(filename, "").unwrap();
        let contents = "";
        return contents;
    }

    match input[0] {
        "open_file" => {
            let path = input[1].clone();
            let mut editor = BufferEditor {
                buffer: Buffer::new(),
            };
            let file_contents =
                std::fs::read_to_string(path).unwrap_or_else(|_| create_file(path).to_string());
            editor.buffer.text = file_contents.clone();
            buffers.insert(String::from(path), editor);
            let buffer = buffers.get_mut(path).unwrap();
            run_game(
                buffer,
                GameSettings::new().tick_duration(Duration::from_millis(25)),
            )?;
            let buffer = buffers.get_mut(path).unwrap();
            std::fs::write(path, &buffer.buffer.text).unwrap();
        }
        "open" => {
            let buffer_name = input[1].trim();
            if !buffers.contains_key(buffer_name) {
                buffers.insert(
                    String::from(buffer_name),
                    BufferEditor {
                        buffer: Buffer::new(),
                    },
                );
            }
            let open_buffer = buffers.get_mut(buffer_name).unwrap();
            run_game(
                open_buffer,
                GameSettings::new().tick_duration(Duration::from_millis(25)),
            )?;
            // println!("{}", &open_buffer.buffer.text);
            // println!("{}", buffers.get_mut(buffer_name).unwrap().buffer.text);
        }
        "search" => {
            todo!()
        }
        "copy_into" => {
            todo!()
        }
        "cut_into" => {
            let (from_buffer, from_line): (&mut BufferEditor, usize) = {
                let (a, b) = input
                    .get(1)
                    .expect("Please specify from buffer")
                    .trim()
                    .rsplit_once(':')
                    .expect("Please specify line number");
                (
                    buffers.get_mut(a).expect("No such from buffer"),
                    b.parse().expect("Not a number"),
                )
            };
            let line = from_buffer
                .buffer
                .text
                .lines()
                .nth(from_line)
                .expect("No such line")
                .to_string();
            from_buffer.buffer.text = {
                let mut a = from_buffer.buffer.text.lines().collect::<Vec<&str>>();
                a.remove(from_line);
                a.join("\n")
            };

            let (to_buffer, to_line): (&mut BufferEditor, usize) = {
                let (a, b) = input
                    .get(2)
                    .expect("Please specify to buffer")
                    .trim()
                    .rsplit_once(':')
                    .expect("Please specify line number");
                (
                    buffers.get_mut(a).expect("No such to buffer"),
                    b.parse().expect("Not a number"),
                )
            };
            to_buffer.buffer.text = {
                let mut a = to_buffer.buffer.text.lines().collect::<Vec<&str>>();
                a.insert(to_line, line.as_str());
                a.join("\n")
            };
        }
        "buffer_from_command" => {
            todo!()
        }
        _ => {
            println!("Command not recognised!");
        }
    }

    Ok(())
}

use rustyline::error::ReadlineError;
use rustyline::Editor;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Welcome to BuffeRS. ");

    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new()?;
    let mut buffers: HashMap<String, BufferEditor> = HashMap::new();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                run_command(&line, &mut buffers)?;
                rl.add_history_entry(line.as_str());
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}
