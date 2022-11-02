// lock.rs
// Locking routines
// Stephen Marz
// 26 Apr 2020

use crate::syscall::syscall_sleep;
use core::arch::asm;

pub const DEFAULT_LOCK_SLEEP: usize = 10000;
#[repr(u32)]
pub enum MutexState {
	Unlocked = 0,
	Locked = 1
}

#[repr(C)]
pub struct Mutex {
	state: MutexState
}

impl<'a> Mutex {
	pub const fn new() -> Self {
		Self { state: MutexState::Unlocked }
	}

	pub fn val(&'a self) -> &'a MutexState {
		&self.state
	}

	/// Try to lock the Mutex. If the mutex is already locked, this function returns false, otherwise it will return true if the mutex was acquired.
	pub fn try_lock(&mut self) -> bool {
		unsafe {
			let state: u64; // MutexState;
			// llvm_asm!("amoswap.w.aq $0, $1, ($2)\n" : "=r"(state) : "r"(1), "r"(self) :: "volatile");
 			asm!("amoswap.w.aq {0}, {1}, ({2})\n", out(reg)state, in(reg)1, in(reg)self);
			match state {
				// amoswap returns the OLD state of the lock.  If it was already locked, we didn't acquire it.
				// MutexState::Locked => false,
				// MutexState::Unlocked => true
				0 => true,
				_ => false
			}
		}
	}

	/// Do NOT sleep lock inside of an interrupt context!
	/// Never use a sleep lock for the process list. Sleeping requires
	/// the process list to function, so you'll deadlock if you do.
	pub fn sleep_lock(&mut self) {
		while !self.try_lock() {
			syscall_sleep(DEFAULT_LOCK_SLEEP);
		}
	}

	/// Can safely be used inside of an interrupt context.
	pub fn spin_lock(&mut self) {
		while !self.try_lock() {}
	}

	/// Unlock a mutex without regard for its previous state.
	pub fn unlock(&mut self) {
		unsafe {
			asm!("amoswap.w.rl zero, zero, ({0})", in(reg)self);
		}
	}
}
