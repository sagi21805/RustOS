use core::clone;

/// This module contains very basic code that helps to interface and create initial page table
///
/// The more advanced code that will be used in the future to allocate table will be in the kernel
///
/// --------------------------------------------------------------------------------------------------
use super::super::super::registers::cr3::get_current_page_table;
use super::address_types::{PhysicalAddress, VirtualAddress};
use crate::flag;
use common::constants::enums::PageSize;
use common::constants::values::{
    BIG_PAGE_SIZE, PAGE_DIRECTORY_ENTRIES, REGULAR_PAGE_ALIGNMENT, REGULAR_PAGE_SIZE,
};
use core::ptr;

// Just a wrapper for the flags for easier use
#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct PageEntryFlags(u64);

impl PageEntryFlags {
    #[inline]
    pub const fn new() -> Self {
        Self(0)
    }
    pub const fn table_flags() -> Self {
        PageEntryFlags::new()
            .set_chain_present()
            .set_chain_writable()
            .set_chain_is_table()
    }
    pub const fn huge_page_flags() -> Self {
        PageEntryFlags::new()
            .set_chain_present()
            .set_chain_writable()
            .set_chain_huge_page()
    }
    pub const fn regular_page_flags() -> Self {
        PageEntryFlags::new()
            .set_chain_present()
            .set_chain_writable()
    }
    flag!(present, 0);
    flag!(writable, 1);
    flag!(usr_access, 2);
    flag!(write_through_cache, 3);
    flag!(disable_cache, 4);
    flag!(huge_page, 7);
    flag!(global, 8);
    flag!(full, 9);
    flag!(is_table, 10);
    flag!(not_executable, 63);
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    #[inline]
    pub(crate) const fn empty() -> Self {
        Self(0)
    }

    // Is this page present?
    flag!(present, 0);

    // Is this page writable?
    flag!(writable, 1);

    // Can this page be accessed from user mode
    flag!(usr_access, 2);

    // Writes go directly to memory
    flag!(write_through_cache, 3);

    // Disable cache for this page
    flag!(disable_cache, 4);

    // This flag can help identifying if an entry is the last one, or it is pointing to another directory
    // Is this page points to a custom memory address and not a page table?
    flag!(huge_page, 7);

    // Page isn’t flushed from caches on address space switch (PGE bit of CR4 register must be set)
    flag!(global, 8);

    // mark a table as full
    flag!(full, 9);

    // This entry points to a table
    flag!(is_table, 10);

    // This page is holding data and is not executable
    flag!(not_executable, 63);

    pub const fn set_flags(&mut self, flags: PageEntryFlags) {
        self.0 &= 0x0000_fffffffff_000; // zero out all previous flags.
        self.0 |= flags.as_u64(); // set new flags;
    }

    #[inline]
    /// Map a frame to the page table entry while checking flags and frame alignment but **not** the ownership of the frame address
    /// This function **will** set the entry as present even if it was not specified in the flags.
    ///
    /// # Parameters
    ///
    /// - `frame`: The physical address of the mapped frame
    ///
    /// # Interrupts
    /// This function will raise a PAGE_FAULT if the entry is already mapped
    ///
    /// # Safety
    /// The `frame` address should not be used by anyone except the corresponding virtual address,
    /// and should be marked owned by it in a memory allocator
    pub const unsafe fn map_unchecked(&mut self, frame: PhysicalAddress, flags: PageEntryFlags) {
        if !self.present() && frame.is_aligned(REGULAR_PAGE_ALIGNMENT) {
            self.set_flags(flags);
            self.set_present();
            self.0 |= (frame.as_usize() as u64 & 0x0000_fffffffff_000);
        } else {
            todo!(
                "Page is already mapped, raise a page fault when interrupt descriptor table is initialized"
            );
        }
    }

    #[inline]
    /// Return the physical address that is mapped by this entry, if this entry is not mapped, return None.
    pub const fn mapped_address(&self) -> Option<PhysicalAddress> {
        if self.present() {
            unsafe { Some(self.mapped_address_unchecked()) }
        } else {
            None
        }
    }

    #[inline]
    pub const unsafe fn mapped_address_unchecked(&self) -> PhysicalAddress {
        unsafe { PhysicalAddress::new_unchecked((self.0 & 0x0000_fffffffff_000) as usize) }
    }

    #[inline]
    /// Return the physical address mapped by this table as a reference into a page table.
    ///
    /// This method assumes all page tables are identity mapped.
    pub fn as_table_mut(&self) -> Option<&mut PageTable> {
        if !self.huge_page() && self.is_table() {
            match self.mapped_address() {
                Some(mapped_address) => unsafe {
                    Some(&mut *mapped_address.as_mut_ptr::<PageTable>())
                },
                None => None,
            }
        } else {
            None
        }
    }

    #[inline]
    #[allow(unsafe_op_in_unsafe_fn)]
    pub unsafe fn as_table_mut_unchecked(&self) -> &mut PageTable {
        &mut *self.mapped_address_unchecked().as_mut_ptr::<PageTable>()
    }

    #[inline]
    #[allow(unsafe_op_in_unsafe_fn)]
    pub unsafe fn as_table_unchecked(&self) -> &PageTable {
        &*self.mapped_address_unchecked().as_ptr::<PageTable>()
    }

    // #[inline]
    // #[allow(unsafe_op_in_unsafe_fn)]
    // unsafe fn as_table_unchecked(&self) -> &PageTable {
    //     core::mem::transmute::<PhysicalAddress, &PageTable>(self.mapped_address_unchecked())
    // }

    #[inline]
    pub fn as_u64(&self) -> u64 {
        self.0
    }

    /// Return a reference to the parent page table of this entry, this is a physical address meant to be accessed with the identity page table
    pub fn parent_page_table_unchecked(&self) -> &'static mut PageTable {
        unsafe {
            &mut *(((self as *const Self as usize) & (usize::MAX - (REGULAR_PAGE_SIZE - 1)))
                as *mut PageTable)
        }
    }
}

#[repr(C)]
#[repr(align(4096))]
#[derive(Debug)]
pub struct PageTable {
    pub entries: [PageTableEntry; PAGE_DIRECTORY_ENTRIES],
}

impl PageTable {
    #[inline]
    pub const unsafe fn from_ptr(page_table_ptr: usize) -> &'static mut PageTable {
        unsafe { &mut *(page_table_ptr as *mut PageTable) }
    }

    #[inline]
    #[allow(unsafe_op_in_unsafe_fn)]
    pub unsafe fn empty_from_ptr(page_table_ptr: usize) -> &'static mut PageTable {
        // core::ptr::write_bytes(page_table_ptr as *mut u8, 0, PAGE_TABLE_SIZE);
        unsafe {
            for i in 0..PAGE_DIRECTORY_ENTRIES {
                ptr::write_volatile((page_table_ptr as *mut u64).add(i), 0);
            }
        }
        &mut *(page_table_ptr as *mut PageTable)
    }

    #[inline]
    pub const fn empty() -> Self {
        Self {
            entries: [const { PageTableEntry::empty() }; PAGE_DIRECTORY_ENTRIES],
        }
    }

    #[inline]
    pub fn address(&self) -> VirtualAddress {
        unsafe { VirtualAddress::new_unchecked(self as *const Self as usize) }
    }

    #[inline]
    pub fn find_available_page(&self, page_size: PageSize) -> Option<VirtualAddress> {
        for (i4, forth_entry) in self.entries.iter().enumerate() {
            if !forth_entry.present() {
                continue;
            }

            let third_table = unsafe { forth_entry.as_table_unchecked() };

            for (i3, third_entry) in third_table.entries.iter().enumerate() {
                if !third_entry.present() {
                    return Some(VirtualAddress::from_indexes(i4, i3, 0, 0));
                }

                if third_entry.huge_page()
                    || !matches!(page_size, PageSize::Big | PageSize::Regular)
                {
                    continue;
                }

                let second_table = unsafe { third_entry.as_table_unchecked() };

                for (i2, second_entry) in second_table.entries.iter().enumerate() {
                    if !second_entry.present() {
                        return Some(VirtualAddress::from_indexes(i4, i3, i2, 0));
                    }

                    if second_entry.huge_page() || !matches!(page_size, PageSize::Regular) {
                        continue;
                    }

                    let first_table = unsafe { second_entry.as_table_unchecked() };

                    for (i1, first_entry) in first_table.entries.iter().enumerate() {
                        if !first_entry.present() {
                            return Some(VirtualAddress::from_indexes(i4, i3, i2, i1));
                        }
                    }
                }
            }
        }
        None
    }
}
