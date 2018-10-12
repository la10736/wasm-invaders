#[cfg(test)]
extern crate rstest;
extern crate cfg_if;
extern crate wasm_bindgen;
extern crate rs8080;
#[cfg(target_arch = "wasm32")]
extern crate js_sys;
#[cfg(not(target_arch = "wasm32"))]
extern crate rand;

#[macro_use]
extern crate log;

mod utils;
mod si;

use std::rc::Rc;
use std::io::Write;
use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;

use rs8080::{
    cpu::{Cpu as Cpu8080, IrqCmd, CpuError},
    hook::NoneHook,
};

use si::{memory::{VRAM_SIZE, ROM_SIZE, SIMmu}, io::{IO, Ev}};

const W: u32 = 256;
const H: u32 = 224;

type Cpu = Cpu8080<SIMmu, Rc<IO>, Rc<IO>, NoneHook>;

#[wasm_bindgen]
pub struct SpaceInvaders {
    cpu: Cpu,
    io: Rc<IO>,
    clocks: u64,
    frames: u64,
}

#[wasm_bindgen]
pub struct Game {
    width: u32,
    height: u32,
    vram: [u8; VRAM_SIZE],
}

#[wasm_bindgen]
impl Game {
    pub fn new() -> Self {
        Self { width: W, height: H, vram: [0; VRAM_SIZE] }
    }

    pub fn vram(&self) -> *const u8 {
        self.vram.as_ptr()
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn space_invaders(&mut self) -> SpaceInvaders {
        let mut rom = [0; ROM_SIZE];
        load_rom(&mut rom);
        let mmu = SIMmu::new(rom.into(), self.vram.as_mut_ptr().into());
        let si_io = IO::default()
            .change_lives(3)
            .coin_info_set(true)
            .lower_bonus_life(true);

        let io = Rc::new(si_io);

        let cpu = Cpu::new(mmu, io.clone(), io.clone(), Default::default());

        SpaceInvaders { cpu, io, clocks: 0, frames: 1 }
    }

    pub fn name(&self) -> String {
        format!("Space Invaders")
    }
}

const CLOCK: u64 = 2_000_000;
const CLOCKS_PER_HALF_FRAME: u64 = CLOCK / 120;
const CLOCKS_PER_FRAME: u64 = CLOCKS_PER_HALF_FRAME * 2;

#[wasm_bindgen]
impl SpaceInvaders {
    pub fn next_frame(&mut self) {
        let done_frame = self.frames * CLOCKS_PER_FRAME;
        let next_half = done_frame + CLOCKS_PER_HALF_FRAME;

        self.run_till(done_frame).unwrap();
        self.cpu.irq(IrqCmd::Irq1).unwrap();

        self.run_till(next_half).unwrap();
        self.cpu.irq(IrqCmd::Irq2).unwrap();

        self.frames += 1;
    }

    pub fn coin(&self, pressed: bool) {
        self.io.ui_event(Ev::Coin, pressed);
    }

    pub fn play(&self, pressed: bool) {
        self.io.ui_event(Ev::P1Start, pressed);
    }

    pub fn left(&self, pressed: bool) {
        self.io.ui_event(Ev::P1Left, pressed);
    }

    pub fn right(&self, pressed: bool) {
        self.io.ui_event(Ev::P1Right, pressed);
    }

    pub fn shoot(&self, pressed: bool) {
        self.io.ui_event(Ev::P1Shoot, pressed);
    }
}

impl SpaceInvaders {
    fn run_till(&mut self, clocks: u64) -> Result<(), CpuError> {
        while self.clocks < clocks {
            self.clocks += self.cpu.run()? as u64;
        }
        Ok(())
    }
}

fn load_rom(rom: &mut [u8]) {
    let data = [include_bytes!("../roms/invaders.h"), include_bytes!("../roms/invaders.g"),
        include_bytes!("../roms/invaders.f"), include_bytes!("../roms/invaders.e")].into_iter();

    for (bytes, mut chunk) in data.zip(rom.chunks_mut(ROM_SIZE / 4)) {
        chunk.write(*bytes).unwrap();
    }
}

#[cfg(target_arch = "wasm32")]
fn random(level: f64) -> bool {
    js_sys::Math::random() < level
}

//Js interface
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console)]
    fn log(msg: &str);

    #[wasm_bindgen(js_namespace = performance)]
    fn now() -> f64;

    #[wasm_bindgen(js_namespace = console)]
    fn time(name: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn timeEnd(name: &str);
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(unused_variables)]
#[allow(dead_code)]
#[allow(non_snake_case)]
mod mock {
    pub fn log(msg: &str) {
        println!("{}", msg)
    }

    pub fn now() -> f64 {
        0.0
    }

    pub fn time(name: &str) {}

    pub fn timeEnd(name: &str) {}

    fn random(level: f64) -> bool {
        ::rand::random::<f64>() < level
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(unused_imports)]
use mock::*;


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_execute_first_1000_frames() {
        let mut game = Game::new();
        let mut si = game.space_invaders();

        for _i in 0..1000 {
            si.next_frame();
        }
    }
}


cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}
