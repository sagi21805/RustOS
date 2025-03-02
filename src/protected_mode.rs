use core::{arch::asm, option};

use crate::constants::KERNEL_START;
// use crate::global_descritor_table::GlobalDescriptorTable;

static GDT: GdtProtectedMode = GdtProtectedMode::new();

#[repr(C)]
pub struct GdtProtectedMode {
    zero: u64,
    code: u64,
    data: u64,
}

impl GdtProtectedMode {
    const fn new() -> Self {
        let limit = {
            let limit_low = 0xffff;
            let limit_high = 0xf << 48;
            limit_high | limit_low
        };
        let access_common = {
            let present = 1 << 47;
            let user_segment = 1 << 44;
            let read_write = 1 << 41;
            present | user_segment | read_write
        };
        let protected_mode = 1 << 54;
        let granularity = 1 << 55;
        let base_flags = protected_mode | granularity | access_common | limit;
        let executable = 1 << 43;
        Self {
            zero: 0,
            code: base_flags | executable,
            data: base_flags,
        }
    }

    fn clear_interrupts_and_load(&'static self) {
        let pointer = GdtPointer {
            base: self,
            limit: (3 * size_of::<u64>() - 1) as u16,
        };

        unsafe {
            asm!("cli", "lgdt [{}]", in(reg) &pointer, options(readonly, nostack, preserves_flags));
        }
    }
}

#[repr(C, packed(2))]
pub struct GdtPointer {
    /// Size of the DT.
    pub limit: u16,
    /// Pointer to the memory region containing the DT.
    pub base: *const GdtProtectedMode,
}


//////////////////////////////////////////////////
#[link_section = ".second_stage"]
#[cfg(feature = "stage-1-2")]
fn set_cr0(val: u32) {
    unsafe {
        asm!(
            "mov cr0, {0:e}",
            in(reg) val,
            options(nostack, preserves_flags)
        )
    }
}

#[link_section = ".second_stage"]
#[cfg(feature = "stage-1-2")]
fn get_cr0() -> u32 {
    let cr0: u32;
    unsafe {
        asm!(
            "mov {0:e}, cr0",
            out(reg) cr0,
            options(nomem, nostack, preserves_flags)    
        )
    }
    return cr0;
}


#[link_section = ".second_stage"]
#[cfg(feature = "stage-1-2")]
pub fn enter_unreal_mode() {

    // This is still a 16-bit instruction mode but with 32-bit access

    let ds: u16;
    let ss: u16;
    unsafe {
        asm!("mov {0:x}, ds", out(reg) ds, options(nomem, nostack, preserves_flags));
        asm!("mov {0:x}, ss", out(reg) ss, options(nomem, nostack, preserves_flags));
    }

    GDT.clear_interrupts_and_load(); // Also disables interrupts

    unsafe {
        // set protected mode bits
        let cr0 = get_cr0();
        set_cr0(cr0 | 1);        
        asm!(
            "mov {0}, 0x10",
            "mov ds, {0}",
            "mov es, {0}",
            "mov fs, {0}",
            "mov gs, {0}",
            "mov ss, {0}", 
            out(reg) _
        );
        set_cr0(cr0); // back to realmode
        // asm!(
        //     "jmp 2f",
        //     "2:",
        //     options(nostack, preserves_flags)
        // ); // Small jump to ensure CR0 takes effect
        
        asm!("mov ds, {0:x}", in(reg) ds, options(nostack, preserves_flags));
        asm!("mov ss, {0:x}", in(reg) ss, options(nostack, preserves_flags));
        asm!("sti");
    }
}

#[link_section = ".second_stage"]
#[cfg(feature = "stage-1-2")]
pub fn enter_protected_mode(entry_point: *const u8) {
    unsafe { asm!("cli") };
     let cr0 = get_cr0();
     set_cr0(cr0 | 1); // toggle protected mode (GDT already loaded from unreal mode)
     unsafe {
        asm!(
            // align the stack
            "and esp, 0xffffff00",
            // push arguments
            "push {entry_point:e}",
            entry_point = in(reg) entry_point as u32,
        );
        asm!("ljmp $0x8, $2f", "2:", options(att_syntax));
        asm!(
            ".code32",

            // reload segment registers
            "mov {0}, 0x10",
            "mov ds, {0}",
            "mov es, {0}",
            "mov ss, {0}",

            // jump to third stage
            "pop {1}",
            "call {1}",

            // enter endless loop in case third stage returns
            "2:",
            "jmp 2b",
            out(reg) _,
            out(reg) _,
        );
    }
}