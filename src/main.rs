use std::{char, env, fs, mem, process};
use uefi::guid::{Guid, GuidKind};

fn main() {
    let path = match env::args().nth(1) {
        Some(some) => some,
        None => {
            eprintln!("smmstore [file]");
            process::exit(1);
        }
    };

    let data = fs::read(path)
        .expect("failed to read file");
    
    let mut i = 0;
    while i + 8 <= data.len() {
        let (keysz, valsz) = unsafe {
            let ptr = data.as_ptr().add(i) as *const u32;
            i += 8;
            (*ptr as usize, *ptr.add(1) as usize)
        };

        // No more entries
        if keysz == 0 || keysz == 0xffff_ffff {
            break;
        }

        // Data too short
        if i + keysz + valsz >= data.len() {
            break;
        }

        if keysz > mem::size_of::<Guid>() {
            let (guid, varname, value) = unsafe {
                let ptr = data.as_ptr().add(i);
                i += keysz + valsz + 1;
                (
                    *(ptr as *const Guid),
                    ptr.add(mem::size_of::<Guid>()) as *const u16,
                    ptr.add(keysz)
                )
            };

            print!("\x1B[1m");
            for j in 0..keysz - mem::size_of::<Guid>() {
                unsafe {
                    let w = *varname.add(j);
                    if w == 0 {
                        break;
                    }
                    if let Some(c) = char::from_u32(w as u32) {
                        print!("{}", c);
                    }
                }
            }
            println!(": {}\x1B[0m", valsz);

            for row in 0..(valsz + 15)/16 {
                print!("{:04X}:", row * 16);
                for col in 0..16 {
                    let j = row * 16 + col;
                    if j < valsz {
                        print!(" {:02X}", unsafe { *value.add(j) });
                    }
                }
                println!();
            }
        }

        i = (i + 3) & !3;
    }
}
