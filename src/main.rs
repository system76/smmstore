use std::{char, env, fs, mem, process, slice};
use std::collections::BTreeMap;
use uefi::guid::Guid;

mod fvb;

fn deserialize_v2(data: &[u8]) -> BTreeMap::<&[u8], &[u8]> {
    let mut compact = BTreeMap::<&[u8], &[u8]>::new();

    // FVB header + variable store header
    let mut i = mem::size_of::<fvb::FvbHeader>()
        + mem::size_of::<fvb::VariableStoreHeader>();

    while i + mem::size_of::<fvb::AuthenticatedVariableHeader>() <= data.len() {
        let var_data = &data[i..(i + mem::size_of::<fvb::AuthenticatedVariableHeader>())];
        let variable = plain::from_bytes::<fvb::AuthenticatedVariableHeader>(var_data).unwrap();

        // XXX: What's the "correct" way to check this in SMMSTOREv2?
        if variable.start_id != fvb::VARIABLE_START_ID {
            // Invalid variable
            break;
        }

        if variable.name_size == 0 {
            // No more entries
            break;
        }

        i += mem::size_of::<fvb::AuthenticatedVariableHeader>();

        let name_size = variable.name_size as usize;
        let data_size = variable.data_size as usize;

        if i + name_size + data_size >= data.len() {
            // Data too short
            break;
        }

        let ptr = unsafe { data.as_ptr().add(i) };
        unsafe {
            compact.insert(
                slice::from_raw_parts(
                    ptr,
                    name_size
                ),
                slice::from_raw_parts(
                    ptr.add(name_size),
                    data_size
                )
            );
        }

        // Move to end of data
        i += name_size + data_size;
        // Align to 4 bytes
        i = (i + 3) & !3;
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
    if header.is_valid() {
        println!("Detected SMMSTOREv2 data");
        let compact = deserialize_v2(&data);

        for (key, value) in compact.iter() {
            print!("\x1B[1m");
            let mut j = 0;
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

            for row in 0..(value.len() + 15) / 16 {
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
    } else {
        println!("Assuming SMMSTOREv1 data");
        let compact = deserialize_v1(&data);

        for (key, value) in compact.iter() {
            if key.len() > mem::size_of::<Guid>() && !value.is_empty() {
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
}
