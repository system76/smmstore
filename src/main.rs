use std::{char, env, fs, mem, process, slice};
use std::collections::BTreeMap;
use uefi::guid::Guid;

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

    let mut compact = BTreeMap::<&[u8], &[u8]>::new();

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

        let ptr = unsafe { data.as_ptr().add(i) };
        unsafe {
            compact.insert(
                slice::from_raw_parts(
                    ptr,
                    keysz
                ),
                slice::from_raw_parts(
                    ptr.add(keysz),
                    valsz
                )
            );
        }

        i += keysz + valsz + 1;
        i = (i + 3) & !3;
    }

    for (key, value) in compact.iter() {
        if key.len() > mem::size_of::<Guid>() && !value.is_empty() {
            let (_guid, _varname) = unsafe {
                let ptr = key.as_ptr();
                (
                    *(ptr as *const Guid),
                    ptr.add(mem::size_of::<Guid>()) as *const u16
                )
            };

            print!("\x1B[1m");
            let mut j = mem::size_of::<Guid>();
            while j + 1 < key.len() {
                let w =
                    (key[j] as u16) |
                    (key[j + 1] as u16) << 8;
                if w == 0 {
                    break;
                }
                if let Some(c) = char::from_u32(w as u32) {
                    print!("{}", c);
                }
                j += 2;
            }
            println!(": {}\x1B[0m", value.len());

            for row in 0..(value.len() + 15)/16 {
                print!("{:04X}:", row * 16);
                for col in 0..16 {
                    let j = row * 16 + col;
                    if j < value.len() {
                        print!(" {:02X}", value[j]);
                    }
                }
                println!();
            }
        }
    }
}
