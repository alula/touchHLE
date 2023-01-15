//! `stdio.h`

use crate::dyld::{export_c_func, FunctionExports};
use crate::fs::{GuestOpenOptions, GuestPath};
use crate::mem::{ConstPtr, ConstVoidPtr, GuestUSize, MutPtr, MutVoidPtr, Ptr, SafeRead};
use crate::Environment;
use std::collections::HashMap;
use std::io::{Read, Write};

pub mod printf;

#[derive(Default)]
pub struct State {
    files: HashMap<MutPtr<FILE>, FileHostObject>,
}

const EOF: i32 = -1;

#[allow(clippy::upper_case_acronyms)]
struct FILE {
    _filler: u8,
}
unsafe impl SafeRead for FILE {}

struct FileHostObject {
    file: std::fs::File,
}

fn fopen(env: &mut Environment, filename: ConstPtr<u8>, mode: ConstPtr<u8>) -> MutPtr<FILE> {
    let mut options = GuestOpenOptions::new();
    // all valid modes are UTF-8
    match env.mem.cstr_at_utf8(mode) {
        "r" | "rb" => options.read(),
        "r+" | "rb+" | "r+b" => options.read().append(),
        "w" | "wb" => options.write().create().truncate(),
        "w+" | "wb+" | "w+b" => options.write().create().truncate().read(),
        "a" | "ab" => options.append().create(),
        "a+" | "ab+" | "a+b" => options.append().create().read(),
        // Modern C adds 'x' but that's not in the documentation so presumably
        // iPhone OS did not have it.
        other => panic!("Unexpected fopen() mode {:?}", other),
    };

    match env
        .fs
        .open_with_options(GuestPath::new(&env.mem.cstr_at_utf8(filename)), options)
    {
        Ok(file) => {
            let host_object = FileHostObject { file };
            let file_ptr = env.mem.alloc_and_write(FILE { _filler: 0 });
            env.libc_state.stdio.files.insert(file_ptr, host_object);
            log_dbg!("fopen({:?}, {:?}) => {:?}", filename, mode, file_ptr);
            file_ptr
        }
        Err(()) => {
            // TODO: set errno
            log!(
                "Warning: fopen({:?}, {:?}) failed, returning NULL",
                filename,
                mode
            );
            Ptr::null()
        }
    }
}

fn fread(
    env: &mut Environment,
    buffer: MutVoidPtr,
    item_size: GuestUSize,
    n_items: GuestUSize,
    file_ptr: MutPtr<FILE>,
) -> GuestUSize {
    let file = env.libc_state.stdio.files.get_mut(&file_ptr).unwrap();
    let total_size = item_size.checked_mul(n_items).unwrap();
    let buffer_slice = env.mem.bytes_at_mut(buffer.cast(), total_size);
    // This does actually have exactly the behaviour that the C standard allows
    // and most implementations provide. There's no requirement that partial
    // objects should not be written to the buffer, and perhaps some app will
    // rely on that. The file position also does not need to be rewound!
    let bytes_read = file.file.read(buffer_slice).unwrap_or(0);
    let items_read: GuestUSize = (bytes_read / usize::try_from(item_size).unwrap())
        .try_into()
        .unwrap();
    if bytes_read < buffer_slice.len() {
        // TODO: set errno
        log!(
            "Warning: fread({:?}, {:#x}, {:#x}, {:?}) read only {:#x} of requested {:#x} bytes",
            buffer,
            item_size,
            n_items,
            file_ptr,
            total_size,
            bytes_read
        );
    } else {
        log_dbg!(
            "fread({:?}, {:#x}, {:#x}, {:?}) => {:#x}",
            buffer,
            item_size,
            n_items,
            file_ptr,
            items_read
        );
    };
    items_read
}

fn fwrite(
    env: &mut Environment,
    buffer: ConstVoidPtr,
    item_size: GuestUSize,
    n_items: GuestUSize,
    file_ptr: MutPtr<FILE>,
) -> GuestUSize {
    let file = env.libc_state.stdio.files.get_mut(&file_ptr).unwrap();
    let total_size = item_size.checked_mul(n_items).unwrap();
    let buffer_slice = env.mem.bytes_at(buffer.cast(), total_size);
    // Remarks in fread() apply here too.
    let bytes_written = file.file.write(buffer_slice).unwrap_or(0);
    let items_written: GuestUSize = (bytes_written / usize::try_from(item_size).unwrap())
        .try_into()
        .unwrap();
    if bytes_written < buffer_slice.len() {
        // TODO: set errno
        log!(
            "Warning: fwrite({:?}, {:#x}, {:#x}, {:?}) wrote only {:#x} of requested {:#x} bytes",
            buffer,
            item_size,
            n_items,
            file_ptr,
            total_size,
            bytes_written
        );
    } else {
        log_dbg!(
            "fwrite({:?}, {:#x}, {:#x}, {:?}) => {:#x}",
            buffer,
            item_size,
            n_items,
            file_ptr,
            items_written
        );
    };
    items_written
}

fn fclose(env: &mut Environment, file_ptr: MutPtr<FILE>) -> i32 {
    let file = env.libc_state.stdio.files.remove(&file_ptr).unwrap();

    // The actual closing of the file happens implicitly when `file` falls out
    // of scope. The return value is about whether flushing succeeds.
    match file.file.sync_all() {
        Ok(()) => {
            log_dbg!("fclose({:?}) => 0", file_ptr);
            0
        }
        Err(_) => {
            // TODO: set errno
            log!("Warning: fclose({:?}) failed, returning EOF", file_ptr);
            EOF
        }
    }
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(fopen(_, _)),
    export_c_func!(fread(_, _, _, _)),
    export_c_func!(fwrite(_, _, _, _)),
    export_c_func!(fclose(_)),
];
