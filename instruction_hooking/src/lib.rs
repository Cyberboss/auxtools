pub mod disassemble_env;

use std::{any::Any, cell::UnsafeCell, ffi::c_void};

use auxtools::*;
use detour::RawDetour;

#[cfg(windows)]
signatures! {
	execute_instruction => version_dependent_signature!(
		1590.. => "0F B7 48 ?? 8B ?? ?? 8B F1 8B ?? ?? 81 ?? ?? ?? 00 00 0F 87 ?? ?? ?? ??",
		..1590 => "0F B7 48 ?? 8B 78 ?? 8B F1 8B 14 ?? 81 FA ?? ?? 00 00 0F 87 ?? ?? ?? ??"
	)
}

#[cfg(unix)]
signatures! {
	execute_instruction => universal_signature!("0F B7 47 ?? 8B 57 ?? 0F B7 D8 8B 0C ?? 81 F9 ?? ?? 00 00 77 ?? FF 24 8D ?? ?? ?? ??")
}

// stackoverflow copypasta https://old.reddit.com/r/rust/comments/kkap4e/how_to_cast_a_boxdyn_mytrait_to_an_actual_struct/
pub trait InstructionHookToAny: 'static {
	fn as_any(&mut self) -> &mut dyn Any;
}

impl<T: 'static> InstructionHookToAny for T {
	fn as_any(&mut self) -> &mut dyn Any {
		self
	}
}

pub trait InstructionHook: InstructionHookToAny {
	fn handle_instruction(&mut self, ctx: *mut raw_types::procs::ExecutionContext);
}

pub static mut INSTRUCTION_HOOKS: UnsafeCell<Vec<Box<dyn InstructionHook>>> =
	UnsafeCell::new(Vec::new());

extern "C" {
	// Trampoline to the original un-hooked BYOND execute_instruction code
	static mut execute_instruction_original: *const c_void;

	// Our version of execute_instruction. It hasn't got a calling convention rust knows about, so don't call it.
	fn execute_instruction_hook();
}

#[init(full)]
fn instruction_hooking_init() -> Result<(), String> {
	let byondcore = sigscan::Scanner::for_module(BYONDCORE).unwrap();

	find_signatures_result! { byondcore,
		execute_instruction
	}

	unsafe {
		let hook = RawDetour::new(
			execute_instruction as *const (),
			execute_instruction_hook as *const (),
		)
		.map_err(|_| "Couldn't detour execute_instruction")?;

		hook.enable()
			.map_err(|_| "Couldn't enable execute_instruction detour")?;

		execute_instruction_original = std::mem::transmute(hook.trampoline());

		// We never remove or disable the hook, so just forget about it.
		std::mem::forget(hook);
	}

	Ok(())
}

#[shutdown]
fn instruction_hooking_shutdown() {
	unsafe {
		INSTRUCTION_HOOKS.get_mut().clear();
	}
}

// Handles any instruction BYOND tries to execute.
// This function has to leave `*CURRENT_EXECUTION_CONTEXT` in EAX, so make sure to return it.
#[no_mangle]
extern "C" fn handle_instruction(
	ctx: *mut raw_types::procs::ExecutionContext,
) -> *const raw_types::procs::ExecutionContext {
	unsafe {
		for vec_box in &mut *INSTRUCTION_HOOKS.get() {
			vec_box.handle_instruction(ctx);
		}
	}

	ctx
}
