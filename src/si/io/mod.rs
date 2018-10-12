#![allow(dead_code)]

mod shift_register;

use std::cell::RefCell;

use rs8080::{
    io_bus::{InputBus, OutputBus},
    Byte
};
use self::shift_register::ShiftRegister;



const PORT1: u8 = 0x01;
const DEFAULT_PORT1: u8 = 0x01;
const PORT2: u8 = 0x02;
const DEFAULT_PORT2: u8 = 0x00;
const PORT3: u8 = 0x03;

const SR_OFFSET_PORT: u8 = 0x02;
const SR_DATA_PORT: u8 = 0x04;
const SOUND_A_PORT: u8 = 0x03;
const SOUND_B_PORT: u8 = 0x05;
const WATCHDOG_PORT: u8 = 0x06;

const COIN_BIT: u8 = 0x00;
const TILT_BIT: u8 = 0x02;
const P2START_BIT: u8 = 0x01;
const P1START_BIT: u8 = 0x02;
const P1SHOOT_BIT: u8 = 0x04;
const P1LEFT_BIT: u8 = 0x05;
const P1RIGHT_BIT: u8 = 0x06;
const P2SHOOT_BIT: u8 = 0x04;
const P2LEFT_BIT: u8 = 0x05;
const P2RIGHT_BIT: u8 = 0x06;

const LIVES_MASK: u8 = 0x03;
const BONUS_LIFE_MASK: u8 = 0x08;
const COIN_INFO_MASK: u8 = 0x80;

#[derive(Copy, Clone)]
pub enum Ev {
    Coin,
    Tilt,
    P1Start,
    P1Shoot,
    P1Left,
    P1Right,
    P2Start,
    P2Shoot,
    P2Left,
    P2Right,
}

pub struct IO {
    port1: RefCell<u8>,
    port2: RefCell<u8>,
    sr: RefCell<ShiftRegister>,
}

impl IO {
    pub fn new(port1: u8, port2: u8) -> Self {
        IO {
            port1: RefCell::new(port1),
            port2: RefCell::new(port2),
            sr: Default::default(),
        }
    }

    pub fn ui_event(&self, ev: Ev, pressed: bool) {
        let (port, bit, set) = match ev {
            Ev::Coin => (&self.port1, COIN_BIT, !pressed),
            Ev::Tilt => (&self.port2, TILT_BIT, pressed),
            Ev::P1Start => (&self.port1, P1START_BIT, pressed),
            Ev::P1Shoot => (&self.port1, P1SHOOT_BIT, pressed),
            Ev::P1Left => (&self.port1, P1LEFT_BIT, pressed),
            Ev::P1Right => (&self.port1, P1RIGHT_BIT, pressed),
            Ev::P2Start => (&self.port1, P2START_BIT, pressed),
            Ev::P2Shoot => (&self.port2, P1SHOOT_BIT, pressed),
            Ev::P2Left => (&self.port2, P1LEFT_BIT, pressed),
            Ev::P2Right => (&self.port2, P1RIGHT_BIT, pressed),
        };

        if set {
            *port.borrow_mut() |= 0x01 << bit;
        } else {
            *port.borrow_mut() &= !(0x01 << bit);
        }
    }

    pub fn bonus_life(&self) -> u16 {
        match *self.port2.borrow() & BONUS_LIFE_MASK {
            BONUS_LIFE_MASK => 1000,
            _ => 1500,
        }
    }

    pub fn lower_bonus_life(self, value: bool) -> Self {
        let state = mask(*self.port2.borrow(), BONUS_LIFE_MASK, value);
        IO {
            port2: RefCell::new(state),
            ..self
        }
    }

    pub fn coin_info(&self) -> bool {
        *self.port2.borrow() & COIN_INFO_MASK == 0x00
    }

    pub fn coin_info_set(self, value: bool) -> Self {
        let state = mask(*self.port2.borrow(), COIN_INFO_MASK, !value);
        IO {
            port2: RefCell::new(state),
            ..self
        }
    }

    pub fn coin_info_off(self) -> Self {
        self.coin_info_set(false)
    }

    pub fn change_lives(self, lives: u8) -> Self {
        let old = *self.port2.borrow();
        let l = match lives {
            0 | 1 | 2 | 3 => 0,
            4 => 1,
            5 => 2,
            6 => 3,
            _ => 3,
        };
        IO {
            port2: RefCell::new((old & !(LIVES_MASK)) | l),
            ..self
        }
    }

    pub fn lives(&self) -> u8 {
        match *self.port2.borrow() & LIVES_MASK {
            0 => 3,
            1 => 4,
            2 => 5,
            3 => 6,
            _ => unreachable!()
        }
    }
}

fn mask(data: Byte, mask: Byte, set_or_clear: bool) -> Byte {
    match set_or_clear {
        true => data | mask,
        false => data & !mask,
    }
}

impl Default for IO {
    fn default() -> Self {
        Self::new(DEFAULT_PORT1, DEFAULT_PORT2)
    }
}

impl InputBus for IO {
    fn read(&self, id: u8) -> Byte {
        match id {
            PORT1 => *self.port1.borrow(),
            PORT2 => *self.port2.borrow(),
            PORT3 => self.sr.borrow().get(),
            _ => {
                warn!("Ask for unknown port {}", id);
                0xFF
            }
        }
    }
}

impl OutputBus for IO {
    fn send(&self, id: u8, data: Byte) {
        match id {
            SR_DATA_PORT => {
                self.sr.borrow_mut().push(data);
            }
            SR_OFFSET_PORT => {
                self.sr.borrow_mut().set_offset(data);
            }
            SOUND_A_PORT | SOUND_B_PORT => debug!("Write to sound port[{}]={:02x}", id, data),
            WATCHDOG_PORT => debug!("Write to watchdog {:02x}", data),
            _ => warn!("Write to unknown port {}={:02x}", id, data)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rstest::{rstest_parametrize, rstest};

    fn io() -> IO {
        IO::default()
    }

    #[test]
    fn default_state() {
        let io = IO::default();

        assert_eq!(DEFAULT_PORT1, io.read(PORT1));
        assert_eq!(DEFAULT_PORT2, io.read(PORT2));
    }

    #[rstest_parametrize(
    port, event, state, bit, set,
    case(PORT1, Unwrap("Ev::Coin"), true, COIN_BIT, false),
    case(PORT1, Unwrap("Ev::Coin"), false, COIN_BIT, true),
    case(PORT2, Unwrap("Ev::Tilt"), true, TILT_BIT, true),
    case(PORT2, Unwrap("Ev::Tilt"), false, TILT_BIT, false),
    case(PORT1, Unwrap("Ev::P2Start"), true, P2START_BIT, true),
    case(PORT1, Unwrap("Ev::P2Start"), false, P2START_BIT, false),
    case(PORT1, Unwrap("Ev::P1Start"), true, P1START_BIT, true),
    case(PORT1, Unwrap("Ev::P1Start"), false, P1START_BIT, false),
    case(PORT1, Unwrap("Ev::P1Shoot"), true, P1SHOOT_BIT, true),
    case(PORT1, Unwrap("Ev::P1Shoot"), false, P1SHOOT_BIT, false),
    case(PORT1, Unwrap("Ev::P1Left"), true, P1LEFT_BIT, true),
    case(PORT1, Unwrap("Ev::P1Left"), false, P1LEFT_BIT, false),
    case(PORT1, Unwrap("Ev::P1Right"), true, P1RIGHT_BIT, true),
    case(PORT1, Unwrap("Ev::P1Right"), false, P1RIGHT_BIT, false),
    case(PORT2, Unwrap("Ev::P2Shoot"), true, P2SHOOT_BIT, true),
    case(PORT2, Unwrap("Ev::P2Shoot"), false, P2SHOOT_BIT, false),
    case(PORT2, Unwrap("Ev::P2Left"), true, P2LEFT_BIT, true),
    case(PORT2, Unwrap("Ev::P2Left"), false, P2LEFT_BIT, false),
    case(PORT2, Unwrap("Ev::P2Right"), true, P2RIGHT_BIT, true),
    case(PORT2, Unwrap("Ev::P2Right"), false, P2RIGHT_BIT, false),
    )]
    fn should_change_bit(io: IO, port: u8, event: Ev, state: bool, bit: u8, set: bool) {
        io.ui_event(event, state);

        let mask = 0x01 << bit;
        let expected = if set { mask } else { 0x00 };

        assert!(io.read(port) & mask == expected);
    }

    #[rstest_parametrize(
    lives, value,
    case(0, 0),
    case(3, 0),
    case(4, 1),
    case(5, 2),
    case(6, 3),
    case(7, 3),
    )]
    fn change_n_lives(io: IO, lives: u8, value: u8) {
        let io = io.change_lives(lives);

        assert_eq!(io.read(PORT2) & LIVES_MASK, value)
    }

    #[rstest]
    fn ask_for_lives(io: IO) {
        assert_eq!(3, io.lives());

        let io = io.change_lives(12);

        assert_eq!(6, io.lives());
    }

    #[rstest]
    fn change_bonus_life(io: IO) {
        assert_eq!(1500, io.bonus_life());

        let io = io.lower_bonus_life(true);

        assert_eq!(1000, io.bonus_life());

        let io = io.lower_bonus_life(false);

        assert_eq!(1500, io.bonus_life());
    }

    #[rstest]
    fn change_coin_info(io: IO) {
        assert_eq!(true, io.coin_info());

        let io = io.coin_info_off();

        assert_eq!(false, io.coin_info());

        let io = io.coin_info_set(true);

        assert_eq!(true, io.coin_info());

        let io = io.coin_info_set(false);

        assert_eq!(false, io.coin_info());
    }

    #[rstest]
    fn should_implement_shift_register(io: IO) {
        assert_eq!(0x00, io.read(PORT3));

        io.send(SR_DATA_PORT, 0xA5);
        io.send(SR_DATA_PORT, 0xFF);

        assert_eq!(0xFF, io.read(PORT3));

        io.send(SR_OFFSET_PORT, 0x04);

        assert_eq!(0xFA, io.read(PORT3));
    }
}
