use std::ptr;

use rs8080::{
    Byte, Address,
    cpu::{
        Result, CpuError, str_memory, DUMP_MEMORY_COLUMNS
    },
    mmu::Mmu
};


trait MBank: Mmu {
    fn offset(&self) -> usize;
    fn size(&self) -> usize;
    fn contains(&self, address: Address) -> bool {
        let address = address as usize;
        address >= self.offset() && address < (self.offset() + self.size())
    }
    fn address(&self, address: Address) -> usize {
        address as usize - self.offset()
    }
}

pub const ROM_SIZE: usize = 0x2000;
pub const RAM_SIZE: usize = 0x0400;
pub const VRAM_SIZE: usize = 0x1C00;
pub const MIRROR_SIZE: usize = 0xC000;

const ROM_OFFSET: usize = 0x0000;
const RAM_OFFSET: usize = 0x2000;
const VRAM_OFFSET: usize = 0x2400;

const MIRROR_OFFSET: usize = 0x4000;

pub struct Rom {
    data: [Byte; ROM_SIZE],
}

impl From<[Byte; ROM_SIZE]> for Rom {
    fn from(data: [u8; ROM_SIZE]) -> Self {
        Rom {
            data,
        }
    }
}

impl Default for Rom {
    fn default() -> Self {
        Rom { data: [0; ROM_SIZE] }
    }
}

impl MBank for Rom {
    fn offset(&self) -> usize {
        ROM_OFFSET
    }

    fn size(&self) -> usize {
        self.data.len()
    }
}

impl Mmu for Rom {
    fn read_byte(&self, address: Address) -> Result<Byte> {
        Ok(self.data[self.address(address)])
    }

    fn write_byte(&mut self, address: Address, val: Byte) -> Result<()> {
        error!("Try to write in rom [{:04x}]={:02x}", address, val);
        CpuError::memory_write(address, val)
    }

    fn dump(&self) -> String {
        str_memory(&self.data, self.offset(), DUMP_MEMORY_COLUMNS)
    }
}

struct Ram {
    data: [Byte; RAM_SIZE],
}

impl Default for Ram {
    fn default() -> Self {
        Ram { data: [0; RAM_SIZE] }
    }
}

impl MBank for Ram {
    fn offset(&self) -> usize {
        RAM_OFFSET
    }

    fn size(&self) -> usize {
        self.data.len()
    }
}

impl Mmu for Ram {
    fn read_byte(&self, address: Address) -> Result<Byte> {
        Ok(self.data[self.address(address)])
    }

    fn write_byte(&mut self, address: Address, val: Byte) -> Result<()> {
        let address = self.address(address);
        self.data[address] = val;
        Ok(())
    }

    fn dump(&self) -> String {
        str_memory(&self.data, self.offset(), DUMP_MEMORY_COLUMNS)
    }
}

pub struct VRam {
    ptr: *mut Byte,
}

impl From<*mut Byte> for VRam {
    fn from(ptr: *mut Byte) -> Self {
        Self::new(ptr)
    }
}

impl VRam {
    pub fn new(ptr: *mut Byte) -> Self {
        VRam { ptr }
    }
}

impl Default for VRam {
    fn default() -> Self {
        Self::new(ptr::null_mut())
    }
}

impl MBank for VRam {
    fn offset(&self) -> usize {
        VRAM_OFFSET
    }

    fn size(&self) -> usize {
        VRAM_SIZE
    }
}

impl Mmu for VRam {
    fn read_byte(&self, address: Address) -> Result<Byte> {
        let addr = self.address(address);
        let val = unsafe {
            *self.ptr.offset(addr as isize)
        };
        debug!("Read Vram [0x{:04x}]=0x{:02x}", address, val);
        Ok(val)
    }

    fn write_byte(&mut self, address: Address, val: Byte) -> Result<()> {
        debug!("Write Vram [0x{:04x}]=0x{:02x}", address, val);
        let address = self.address(address);
        unsafe { *self.ptr.offset(address as isize) = val }
        Ok(())
    }

    fn dump(&self) -> String {
        format!("No Data!")
    }
}

#[derive(Default)]
struct Mirror;

impl MBank for Mirror {
    fn offset(&self) -> usize {
        MIRROR_OFFSET
    }

    fn size(&self) -> usize {
        MIRROR_SIZE
    }
}

impl Mmu for Mirror {
    fn read_byte(&self, address: Address) -> Result<Byte> {
        error!("Try to read in mirror 0x{:04x}", address);
        CpuError::memory_read(address)
    }

    fn write_byte(&mut self, address: Address, val: Byte) -> Result<()> {
        error!("Try to write in mirror 0x{:04x}", address);
        CpuError::memory_write(address, val)
    }

    fn dump(&self) -> String {
        format!("Just mirror bank!")
    }
}

#[allow(dead_code)]
#[derive(Default)]
pub struct SIMmu {
    rom: Rom,
    ram: Ram,
    vram: VRam,
    mirror: Mirror,
}

impl SIMmu {
    #[allow(dead_code)]
    pub fn new(rom: Rom, vram: VRam) -> SIMmu {
        SIMmu {
            rom,
            vram,
            ..Default::default()
        }
    }

    fn should_ignore_it(&self, address: Address) -> bool {
        return 0x4000 <= address && address < 0x4200
    }
}

impl Mmu for SIMmu {
    fn read_byte(&self, address: Address) -> Result<Byte> {
        if self.rom.contains(address) {
            self.rom.read_byte(address)
        } else if self.ram.contains(address) {
            self.ram.read_byte(address)
        } else if self.vram.contains(address) {
            self.vram.read_byte(address)
        } else if self.mirror.contains(address) {
            self.mirror.read_byte(address)
        } else {
            unreachable!()
        }
    }

    fn write_byte(&mut self, address: Address, val: Byte) -> Result<()> {
        if self.should_ignore_it(address) {
            debug!("Write access to ignore address 0x{:04x} = 0x{:02x}", address, val);
            return Ok(())
        }
        if self.rom.contains(address) {
            self.rom.write_byte(address, val)
        } else if self.ram.contains(address) {
            self.ram.write_byte(address, val)
        } else if self.vram.contains(address) {
            self.vram.write_byte(address, val)
        } else if self.mirror.contains(address) {
            self.mirror.write_byte(address, val)
        } else {
            unreachable!()
        }
    }

    fn dump(&self) -> String {
        format!(r#"Rom:
{}
Ram:
{}
VRam:
{}
Mirror:
{}"#, self.rom.dump(), self.ram.dump(), self.vram.dump(), self.mirror.dump() )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rstest::{rstest_parametrize};
    use rs8080::Address;
    use rs8080::Byte;

    fn zmem() -> SIMmu {
        SIMmu::default()
    }

    mod ram {
        use super::*;

        #[rstest_parametrize(
        address, value,
        case(0x2000, 0xE1),
        case(0x2020, 0xA5),
        case(0x23FF, 0x1A),
        )]
        fn write_and_read(mut zmem: SIMmu, address: Address, value: Byte) {
            zmem.write_byte(address, value).unwrap();

            assert_eq!(Ok(value), zmem.read_byte(address))
        }
    }

    mod rom {
        use super::*;

        #[rstest_parametrize(
        address, value,
        case(0x0000, 0xA2),
        case(0x1203, 0xE1),
        case(0x1FFF, 0x01),
        )]
        fn write_byte_error(mut zmem: SIMmu, address: Address, value: Byte) {
            assert!(zmem.write_byte(address, value).is_err());
        }

        #[rstest_parametrize(
        address, value,
        case(0x0000, 0xA2),
        case(0x1203, 0xE1),
        case(0x1FFF, 0x01),
        )]
        fn read_byte(mut zmem: SIMmu, address: Address, value: Byte) {
            let raw_address = zmem.rom.address(address);
            zmem.rom.data[raw_address] = value;

            assert_eq!(Ok(value), zmem.read_byte(address))
        }
    }

    mod vram {
        use super::*;

        #[rstest_parametrize(
        address, value,
        case(0x2400, 0xE1),
        case(0x2420, 0xA5),
        case(0x2FFF, 0x1A),
        )]
        fn write_and_read(address: Address, value: Byte) {
            let mut vram = [0; VRAM_SIZE];
            let mut mem = SIMmu { vram: VRam::new(vram.as_mut_ptr()), ..Default::default() };

            mem.write_byte(address, value).unwrap();

            assert_eq!(value, mem.read_byte(address).unwrap());
        }
    }

    mod mirror {
        use super::*;

        fn mirror() -> Mirror {
            Mirror::default()
        }

        #[rstest_parametrize(
        address, value,
        case(0x4000, 0xE1),
        case(0x5420, 0xA5),
        case(0xFFFF, 0x1A),
        )]
        fn read_should_return_error(mirror: Mirror, address: Address) {
            assert!(mirror.read_byte(address).is_err());
        }

        #[rstest_parametrize(
        address, value,
        case(0x4000, 0xE1),
        case(0x5420, 0xA5),
        case(0xFFFF, 0x1A),
        )]
        fn write_should_return_error(mut mirror: Mirror, address: Address, value: Byte) {
            assert!(mirror.write_byte(address, value).is_err());
        }
    }

    #[rstest_parametrize(
    address, value,
    case(0x4000, 0xE1),
    case(0x4100, 0xA5),
    case(0x41ff, 0x1A),
    )]
    fn write_should_ignore(mut zmem: SIMmu, address: Address, value: Byte) {
        assert!(zmem.write_byte(address, value).is_ok());
    }

    #[rstest_parametrize(
    address, value,
    case(0x4200, 0xE1),
    case(0xffff, 0x1A),
    )]
    fn write_should_return_error(mut zmem: SIMmu, address: Address, value: Byte) {
        assert!(zmem.write_byte(address, value).is_err());
    }
}
