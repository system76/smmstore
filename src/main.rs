use std::{char, env, fs, mem, process, slice};
use std::collections::BTreeMap;
use uefi::guid::Guid;

mod fvb;

fn deserialize_v2(data: &[u8]) -> BTreeMap::<&[u8], &[u8]> {
    let mut compact = BTreeMap::<&[u8], &[u8]>::new();

    // FVB header + variable store header + ???
    let mut i = mem::size_of::<fvb::FvbHeader>() + mem::size_of::<fvb::VariableStoreHeader>() + 36;
    while i + 8 <= data.len() {
        let (keysz, valsz) = unsafe {
            let ptr = data.as_ptr().add(i) as *const u32;
            i += 8;
            (*ptr as usize, *ptr.add(1) as usize)
        };

        if keysz == 0 || keysz == 0xffff_ffff {
            // No more entries
            break;
        }

        // GUID is not part of key anymore
        if i + mem::size_of::<Guid>() + keysz + valsz >= data.len() {
            // Data too short
            break;
        }

        let ptr = unsafe { data.as_ptr().add(i) };
        unsafe {
            compact.insert(
                slice::from_raw_parts(
                    ptr,
                    mem::size_of::<Guid>() + keysz
                ),
                slice::from_raw_parts(
                    ptr.add(keysz),
                    valsz
                )
            );
        }

        // No more NULL byte, account for GUID
        i += mem::size_of::<Guid>() + keysz + valsz;
        i = (i + 3) & !3;

        // XXX: ?
        i += 36;
    }

    compact
}

fn deserialize_v1(data: &[u8]) -> BTreeMap::<&[u8], &[u8]> {
    let mut compact = BTreeMap::<&[u8], &[u8]>::new();

    let mut i = 0;
    while i + 8 <= data.len() {
        let (keysz, valsz) = unsafe {
            let ptr = data.as_ptr().add(i) as *const u32;
            i += 8;
            (*ptr as usize, *ptr.add(1) as usize)
        };

        if keysz == 0 || keysz == 0xffff_ffff {
            // No more entries
            break;
        }

        if i + keysz + valsz >= data.len() {
            // Data too short
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

    compact
}

fn main() {
    let path = match env::args().nth(1) {
        Some(some) => some,
        None => {
            eprintln!("smmstore [file]");
            process::exit(1);
        }
    };

    let data = fs::read(path).expect("failed to read file");

    let header_data = &data[..mem::size_of::<fvb::FvbHeader>()];
    let header = plain::from_bytes::<fvb::FvbHeader>(header_data).unwrap();
    let compact = if header.is_valid() {
        println!("Detected SMMSTOREv2 data");
        deserialize_v2(&data)
    } else {
        println!("Assuming SMMSTOREv1 data");
        deserialize_v1(&data)
    };

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
