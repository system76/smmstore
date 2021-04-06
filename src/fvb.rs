use plain::Plain;
use uefi::guid;

pub const FVH_SIGNATURE: &[u8; 4] = b"_FVH";
pub const FVH_REVISION: u8 = 0x02;

#[derive(Clone, Debug)]
#[repr(C)]
pub struct FvbHeader {
    pub zero_vector: [u8; 16],
    pub guid: guid::Guid,
    pub volume_length: u64,
    pub signature: [u8; 4],
    pub attributes: u32,
    pub header_length: u16,
    pub checksum: u16,
    pub ext_header_offset: u16,
    pub reserved: u8,
    pub revision: u8,
    pub block_map: [(u32, u32); 2],
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
#[repr(C)]
pub struct VariableStoreHeader {
    pub signature: guid::Guid,
    pub size: u32,
    pub format: u8,
    pub state: u8,
    pub reserved: u16,
    pub reserved1: u32,
}

unsafe impl Plain for VariableStoreHeader {}

impl VariableStoreHeader {
    pub fn is_valid(&self) -> bool {
        self.signature == guid::AUTHENTICATED_VARIABLE_GUID
    }
}

pub const VARIABLE_START_ID: u16 = 0x55AA;

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct AuthenticatedVariableHeader {
    pub start_id: u16,
    pub state: u8,
    pub reserved: u8,
    pub attributes: u32,
    pub monotonic_count: u64,
    pub timestamp: [u8; 16],
    pub pubkey_index: u32,
    pub name_size: u32,
    pub data_size: u32,
    pub vendor_guid: guid::Guid,
}

unsafe impl Plain for AuthenticatedVariableHeader {}
