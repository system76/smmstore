use plain::Plain;
use uefi::guid;

pub const FVH_SIGNATURE: &[u8; 4] = b"_FVH";
pub const FVH_REVISION: u8 = 0x02;

#[derive(Clone, Debug)]
#[repr(C)]
pub struct FvbHeader {
    zero_vector: [u8; 16],
    guid: guid::Guid,
    volume_length: u64,
    signature: [u8; 4],
    attributes: u32,
    header_length: u16,
    checksum: u16,
    ext_header_offset: u16,
    reserved: u8,
    revision: u8,
    block_map: [(u32, u32); 2],
}

unsafe impl Plain for FvbHeader {}

impl FvbHeader {
    pub fn is_valid(&self) -> bool {
        self.zero_vector.iter().all(|&x| x == 0)
            && self.guid == guid::SYSTEM_NV_DATA_FV_GUID
            && self.signature == *FVH_SIGNATURE
            && self.revision == FVH_REVISION
    }
}

#[derive(Copy, Clone, Debug)]
pub struct VariableStoreHeader {
    signature: guid::Guid,
    size: u32,
    format: u8,
    state: u8,
    reserved: u16,
    reserved1: u32,
}

unsafe impl Plain for FvbHeader {}

impl VariableStoreHeader {
    pub fn is_valid(&self) -> bool {
        self.signature == guid::AUTHENTICATED_VARIABLE_GUID
    }
}
