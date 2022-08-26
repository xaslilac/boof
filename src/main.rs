use ::std::env;
use ::std::fs;
use ::std::io;
use ::std::io::BufWriter;
use ::std::io::Read;
use ::std::io::Stdout;
use ::std::io::Write;
use ::std::path::PathBuf;
use ::std::process::exit;

const CHARACTERS: [char; 9] = ['<', '>', '[', ']', '.', ',', '-', '+', '!'];
const TAPE_SIZE: usize = 30000;

#[derive(Debug)]
struct Boof {
	program: Vec<Instruction>,
	debug: bool,
	halt: bool,
	out: BufWriter<Stdout>,
	tape: [u8; TAPE_SIZE],
	d: usize,
	p: usize,
}

impl Default for Boof {
	fn default() -> Self {
		Boof {
			program: vec![],
			debug: false,
			halt: false,
			out: BufWriter::new(io::stdout()),
			tape: [0; TAPE_SIZE],
			d: 0,
			p: 0,
		}
	}
}

impl Boof {
	pub fn start(&mut self) {
		while !self.halt {
			self.tick();
		}
	}

	pub fn tick(&mut self) {
		use Instruction::*;

		let instr = self.program[self.p];
		if self.debug {
			println!("{:0>2x?}", &self.tape[0..30]);
			println!("d: {}, p: {}", self.d, self.p);
			println!("{:?}", instr);
			println!("----------------");
		}
		self.p += 1;

		match instr {
			idp => self.d += 1,
			ddp => self.d -= 1,
			inc => self.tape[self.d] += 1,
			dec => self.tape[self.d] -= 1,
			put => {
				if self.debug {
					self.out.flush().expect("failed to write to stdout");
					write!(
						io::stdout(),
						".: {} (0x{:x})\n",
						self.tape[self.d] as char,
						self.tape[self.d]
					)
					.unwrap();
				} else {
					_ = self.out.write(&[self.tape[self.d]]).unwrap();
				}
			}
			get => {
				self.tape[self.d] = io::stdin()
					.bytes()
					.next()
					.expect("no byte to read")
					.expect("failed to read byte");
			}
			jump(to) => {
				if self.tape[self.d] == 0 {
					self.p = to + 1;
				}
			}
			end(to) => self.p = to,
			halt => self.halt = true,
		};

		if self.p >= self.program.len() {
			self.halt = true;
			return;
		}

		if self.debug {
			std::thread::sleep(std::time::Duration::from_secs_f32(0.1));
		}
	}
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
enum Instruction {
	idp,
	ddp,
	inc,
	dec,
	put,
	get,
	jump(usize),
	end(usize),
	halt,
}

#[derive(Clone, Debug, Default)]
struct OptionsBuilder {
	debug: bool,
	input: Option<String>,
}

#[derive(Clone, Debug)]
struct Options {
	debug: bool,
	input: PathBuf,
}

impl Into<Options> for OptionsBuilder {
	fn into(self) -> Options {
		let input = PathBuf::from(self.input.expect("must provide an input file"));

		Options {
			debug: self.debug,
			input,
		}
	}
}

fn main() -> io::Result<()> {
	use Instruction::*;
	let mut args = env::args().skip(1);
	let mut ob = OptionsBuilder::default();

	while let Some(arg) = args.next() {
		if (arg.len() == 2 && arg.starts_with('-')) || args.len() > 3 && arg.starts_with("--") {
			match arg.as_ref() {
				"-d" | "--debug" => {
					ob.debug = true;
				}
				_ => {
					println!("unrecognized option: {}", arg);
					exit(1);
				}
			}
		} else {
			ob.input = Some(arg);
		}
	}

	let options: Options = ob.into();

	let mut code = fs::read_to_string(options.input)?;
	code.retain(|c| CHARACTERS.contains(&c));
	let mut program = vec![halt; code.len()];
	let mut b_stack: Vec<usize> = vec![];

	for (i, char) in code.chars().enumerate() {
		program[i] = match char {
			'>' => idp,
			'<' => ddp,
			'+' => inc,
			'-' => dec,
			'.' => put,
			',' => get,
			'[' => {
				b_stack.push(i);
				jump(0)
			}
			']' => {
				let b = b_stack.pop().expect("unmatched ]");
				match program[b] {
					jump(_) => {
						program[b] = jump(i);
						end(b)
					}
					_ => unreachable!(),
				}
			}
			'!' => halt,
			_ => unreachable!(),
		}
	}

	let mut boof = Boof::default();
	boof.debug = options.debug;
	boof.program = program;
	boof.start();

	Ok(())
}
