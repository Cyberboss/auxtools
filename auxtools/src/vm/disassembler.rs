use crate::vm::vm as vmhook;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;
use vmhook::Opcode;
use vmhook::Opcode::*;
use vmhook::VType;
use vmhook::VValue;

#[derive(Debug)]
pub struct Dism {
	cursor: Cursor<Vec<u8>>,
}
#[derive(Debug)]
pub struct TempRegister(usize);
#[derive(Debug)]
pub struct LocalRegister(usize);
#[derive(Debug)]
pub struct ArgRegister(usize);
#[derive(Debug)]
pub struct Location(u32);
#[derive(Debug)]
pub struct StringId(u16);

#[derive(Debug)]
pub enum DisOpcode {
	Halt,
	LoadImmediate(TempRegister, VType, VValue),
	LoadArgument(ArgRegister, TempRegister),
	LoadLocal(LocalRegister, TempRegister),
	StoreLocal(TempRegister, LocalRegister),
	GetField(TempRegister, StringId, TempRegister),
	SetField(TempRegister, StringId, TempRegister),
	Add(TempRegister, TempRegister, TempRegister),
	Sub(TempRegister, TempRegister, TempRegister),
	Mul(TempRegister, TempRegister, TempRegister),
	Div(TempRegister, TempRegister, TempRegister),
	LessThan(TempRegister, TempRegister, TempRegister),
	LessOrEqual(TempRegister, TempRegister, TempRegister),
	Equal(TempRegister, TempRegister, TempRegister),
	GreaterOrEqual(TempRegister, TempRegister, TempRegister),
	GreaterThan(TempRegister, TempRegister, TempRegister),
	Jump(Location),
	JumpTrue(TempRegister, Location),
	JumpFalse(TempRegister, Location),
	Push(TempRegister),
	Call(StringId, TempRegister),
	Return(TempRegister),
	Invalid,
}

macro_rules! left_right_result {
	($me:expr, $op:tt) => {{
		let lefti = $me.read_register();
		let righti = $me.read_register();
		let desti = $me.read_register();

		Some($op(
			TempRegister(lefti),
			TempRegister(righti),
			TempRegister(desti),
			))
		}};
}

impl Dism {
	pub fn new(bytecode: Vec<u8>) -> Self {
		Self {
			cursor: Cursor::new(bytecode),
		}
	}

	fn next_opcode(&mut self) -> Opcode {
		Opcode::from(self.next_byte())
	}

	fn next_byte(&mut self) -> u8 {
		self.cursor.read_u8().unwrap()
	}

	fn read_register(&mut self) -> usize {
		self.next_byte() as usize
	}

	fn read_type(&mut self) -> VType {
		self.next_byte() as VType
	}

	fn read_value(&mut self) -> VValue {
		self.cursor.read_u32::<LittleEndian>().unwrap() as VValue
	}

	fn read_short(&mut self) -> u16 {
		self.cursor.read_u16::<LittleEndian>().unwrap()
	}

	pub fn disassemble_one(&mut self) -> Option<DisOpcode> {
		use DisOpcode::*;
		let op = self.next_opcode();
		return match op {
			LOAD_IMMEDIATE => {
				let reg_idx = self.read_register();
				let typ = self.read_type();
				let val = self.read_value();

				Some(LoadImmediate(TempRegister(reg_idx), typ, val))
			}
			LOAD_ARGUMENT => {
				let arg_index = self.read_register();
				let dest_index = self.read_register();

				Some(LoadArgument(
					ArgRegister(arg_index),
					TempRegister(dest_index),
				))
			}
			LOAD_LOCAL => {
				let local_index = self.read_register();
				let dest_index = self.read_register();

				Some(LoadLocal(
					LocalRegister(local_index),
					TempRegister(dest_index),
				))
			}
			STORE_LOCAL => {
				let dest_index = self.read_register();
				let local_index = self.read_register();

				Some(StoreLocal(
					TempRegister(dest_index),
					LocalRegister(local_index),
				))
			}
			GET_FIELD => {
				let source_index = self.read_register();
				let field_name = self.read_short();
				let destination_index = self.read_register();

				Some(GetField(
					TempRegister(source_index),
					StringId(field_name),
					TempRegister(destination_index),
				))
			}
			ADD => left_right_result!(self, Add),
			SUB => left_right_result!(self, Sub),
			MUL => left_right_result!(self, Mul),
			DIV => left_right_result!(self, Div),
			LESS_THAN => left_right_result!(self, LessThan),
			LESS_OR_EQUAL => left_right_result!(self, LessOrEqual),
			EQUAL => left_right_result!(self, Equal),
			GREATER_OR_EQUAL => left_right_result!(self, GreaterOrEqual),
			GREATER_THAN => left_right_result!(self, GreaterThan),
			JUMP => {
				let dest = self.read_value();
				Some(Jump(Location(dest)))
			}
			JUMP_TRUE => {
				let reg = self.read_register();
				let dest = self.read_value();
				Some(JumpTrue(TempRegister(reg), Location(dest)))
			}
			JUMP_FALSE => {
				let reg = self.read_register();
				let dest = self.read_value();
				Some(JumpFalse(TempRegister(reg), Location(dest)))
			}
			PUSH => {
				let arg_idx = self.read_register();
				Some(Push(TempRegister(arg_idx)))
			}
			CALL => {
				let proc_id = self.read_value() as u16;
				let result_register = self.read_register();

				Some(Call(StringId(proc_id), TempRegister(result_register)))
			}
			RETURN => {
				let return_register_id = self.read_register();
				Some(Return(TempRegister(return_register_id)))
			}
			_ => None,
		};
	}

	pub fn disassemble(&mut self) -> Vec<DisOpcode> {
		let mut res: Vec<DisOpcode> = vec![];
		while let Some(op) = self.disassemble_one() {
			res.push(op);
		}
		res
	}
}
