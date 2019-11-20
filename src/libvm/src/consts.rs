pub const PUSH_I: u8 = 0x00;

pub const ADD_I: u8 = 0x0c;
pub const SUB_I: u8 = 0x0d;
pub const MUL_I: u8 = 0x0e;
pub const DIV_I: u8 = 0x0f;
pub const MOD_I: u8 = 0x10;

pub const NE: u8 = 0x11;
pub const EQ: u8 = 0x12;
pub const GT: u8 = 0x13;
pub const LT: u8 = 0x14;
pub const LE: u8 = 0x15;
pub const GE: u8 = 0x16;

pub const CMP_I: u8 = 0x20;

pub const IF_T: u8 = 0xa0;
pub const IF_F: u8 = 0xa1;

pub const IF_NE: u8 = 0xa2;
pub const IF_EQ: u8 = 0xa3;
pub const IF_GT: u8 = 0xa4;
pub const IF_LT: u8 = 0xa5;
pub const IF_LE: u8 = 0xa6;
pub const IF_GE: u8 = 0xa7;

pub const DUP_I: u8 = 0xdf;

pub const GOTO: u8 = 0xc0;

pub const LOAD_I: u8 = 0xfc;

pub const STO_I: u8 = 0xfd;

pub const VIRTUAL: u8 = 0xfe;

pub const RET: u8 = 0xff;

pub fn disassemble_each(val: u8) -> Option<&'static str> {
	match val {
		PUSH_I => Some("push_i"),
		ADD_I => Some("add_i"),
		SUB_I => Some("sub_i"),
		MUL_I => Some("mul_i"),
		DIV_I => Some("div_i"),
		MOD_I => Some("mod_i"),
		CMP_I => Some("cmp_i"),
		NE => Some("ne"),
		EQ => Some("eq"),
		LT => Some("lt"),
		GT => Some("gt"),
		LE => Some("le"),
		GE => Some("ge"),
		IF_T => Some("if_t"),
		IF_F => Some("if_f"),
		IF_NE => Some("if_ne"),
		IF_EQ => Some("if_eq"),
		IF_GT => Some("if_gt"),
		IF_LT => Some("if_lt"),
		IF_LE => Some("if_le"),
		IF_GE => Some("if_ge"),
		DUP_I => Some("dup_i"),
		GOTO => Some("goto"),
		LOAD_I => Some("load_i"),
		STO_I => Some("sto_i"),
		VIRTUAL => Some("virtual"),
		RET => Some("ret"),
		_ => Some("unknown")
	}
}

pub fn disassemble(program: &[u8]) -> String {
	let mut out = String::new();
	let mut program = program.iter().enumerate();
	macro_rules! push_n {
		($n: expr) => {
			for _ in 0..$n {
				out.push(' ');
				out.push_str(&program.next().unwrap().1.to_string());
			}
		};
	}
	while let Some((i, v)) = program.next() {
		let i_str = i.to_string();
		out.push_str("\u{001b}[33m"); // red
		out.push_str(&i_str);
		out.push_str(": ");
		out.push_str("\u{001b}[0m"); // reset
		for _ in 0..(3-i_str.len()) {
			out.push(' ');
		}
		out.push_str("\u{001b}[31m"); // blue
		let in_str = disassemble_each(*v).unwrap();
		out.push_str(in_str);
		for _ in 0..(8-in_str.len()) {
			out.push(' ');
		}
		out.push_str("\u{001b}[0m"); // reset
		match *v {
			PUSH_I => push_n!(4),
			VIRTUAL | GOTO | STO_I | LOAD_I | IF_T..=IF_GE => push_n!(1),
			
			_ => {}
		}
		out.push('\n');
	}
	out
}
