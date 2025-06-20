pub const FIRST_STAGE_OFFSET: u16 = 0x7c00;
pub const MASTER_BOOT_RECORD_OFFSET: u16 = FIRST_STAGE_OFFSET + 446;
pub const SECOND_STAGE_OFFSET: u16 = FIRST_STAGE_OFFSET + 512;
pub const DISK_NUMBER_OFFSET: u16 = FIRST_STAGE_OFFSET;
pub const VGA_BUFFER_PTR: u32 = 0xb8000;
pub const KERNEL_OFFSET: u64 = 0x9000;
pub const IDENTITY_PAGE_TABLE_L4_OFFSET: usize = 0x10000;
pub const IDENTITY_PAGE_TABLE_L3_OFFSET: usize = 0x11000;
pub const IDENTITY_PAGE_TABLE_L2_OFFSET: usize = 0x12000;
pub const TOP_IDENTITY_PAGE_TABLE_L3_OFFSET: usize = 0x13000;
pub const TOP_IDENTITY_PAGE_TABLE_L2_OFFSET: usize = 0x14000;
pub const PAGE_ALLOCATOR_OFFSET: usize = 0x15000;
#[cfg(target_arch = "x86_64")]
pub const PHYSICAL_MEMORY_OFFSET: usize = 0xffff800000000000;
