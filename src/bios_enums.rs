#[allow(dead_code)]

pub enum Interrupts {
    VIDEO = 0x10,
}

pub enum Video {
    SetMode = 0x0,
    DisplayChar = 0xE,
    DisplayStr = 0x13,
}

#[allow(non_camel_case_types)]
pub enum VideoModes {
    /// VGA Common Text Mode ->
    ///
    /// Text resulotion 80x25
    ///
    /// PixelBox resulotion 9x16
    ///
    /// Pixel Resulption 720x400
    VGA_TX_80X25_PB_9X16_PR_720X400 = 0x3,
}

pub enum PacketSize {
    Default = 0x10,
}

#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}