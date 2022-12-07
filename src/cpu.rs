//! CPU emulation.
//!
//! Implemented using the C++ library dynarmic, which is a dynamic recompiler.

use crate::memory::{ConstPtr, Memory, MutPtr, Ptr, SafeRead};

// Import functions from C++
use touchHLE_dynarmic_wrapper::*;

type VAddr = u32;

fn touchHLE_cpu_read_impl<T: SafeRead>(mem: *mut touchHLE_Memory, addr: VAddr) -> T {
    let mem = unsafe { &mut *mem.cast::<Memory>() };
    let ptr: ConstPtr<T> = Ptr::from_bits(addr);
    mem.read(ptr)
}

fn touchHLE_cpu_write_impl<T>(mem: *mut touchHLE_Memory, addr: VAddr, value: T) {
    let mem = unsafe { &mut *mem.cast::<Memory>() };
    let ptr: MutPtr<T> = Ptr::from_bits(addr);
    mem.write(ptr, value)
}

// Export functions for use by C++
#[no_mangle]
extern "C" fn touchHLE_cpu_read_u8(mem: *mut touchHLE_Memory, addr: VAddr) -> u8 {
    touchHLE_cpu_read_impl(mem, addr)
}
#[no_mangle]
extern "C" fn touchHLE_cpu_read_u16(mem: *mut touchHLE_Memory, addr: VAddr) -> u16 {
    touchHLE_cpu_read_impl(mem, addr)
}
#[no_mangle]
extern "C" fn touchHLE_cpu_read_u32(mem: *mut touchHLE_Memory, addr: VAddr) -> u32 {
    touchHLE_cpu_read_impl(mem, addr)
}
#[no_mangle]
extern "C" fn touchHLE_cpu_read_u64(mem: *mut touchHLE_Memory, addr: VAddr) -> u64 {
    touchHLE_cpu_read_impl(mem, addr)
}
#[no_mangle]
extern "C" fn touchHLE_cpu_write_u8(mem: *mut touchHLE_Memory, addr: VAddr, value: u8) {
    touchHLE_cpu_write_impl(mem, addr, value);
}
#[no_mangle]
extern "C" fn touchHLE_cpu_write_u16(mem: *mut touchHLE_Memory, addr: VAddr, value: u16) {
    touchHLE_cpu_write_impl(mem, addr, value);
}
#[no_mangle]
extern "C" fn touchHLE_cpu_write_u32(mem: *mut touchHLE_Memory, addr: VAddr, value: u32) {
    touchHLE_cpu_write_impl(mem, addr, value);
}
#[no_mangle]
extern "C" fn touchHLE_cpu_write_u64(mem: *mut touchHLE_Memory, addr: VAddr, value: u64) {
    touchHLE_cpu_write_impl(mem, addr, value);
}

pub struct Cpu {
    dynarmic_wrapper: *mut touchHLE_DynarmicWrapper,
}

impl Drop for Cpu {
    fn drop(&mut self) {
        unsafe { touchHLE_DynarmicWrapper_delete(self.dynarmic_wrapper) }
    }
}

impl Cpu {
    pub fn new() -> Cpu {
        let dynarmic_wrapper = unsafe { touchHLE_DynarmicWrapper_new() };
        Cpu { dynarmic_wrapper }
    }

    pub fn regs(&self) -> &[u32; 16] {
        unsafe {
            let ptr = touchHLE_DynarmicWrapper_regs_const(self.dynarmic_wrapper);
            &*(ptr as *const [u32; 16])
        }
    }
    pub fn regs_mut(&mut self) -> &mut [u32; 16] {
        unsafe {
            let ptr = touchHLE_DynarmicWrapper_regs_mut(self.dynarmic_wrapper);
            &mut *(ptr as *mut [u32; 16])
        }
    }

    // TODO: this should have a return value so we know why execution ended
    pub fn run(&mut self, mem: &mut Memory) {
        unsafe {
            touchHLE_DynarmicWrapper_run(
                self.dynarmic_wrapper,
                mem as *mut Memory as *mut touchHLE_Memory,
            )
        }
    }
}
