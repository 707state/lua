use core::ffi::c_int;

use crate::runtime::*;

const fn opmode(mm: u8, ot: u8, it: u8, t: u8, a: u8, m: u8) -> u8 {
    (mm << 7) | (ot << 6) | (it << 5) | (t << 4) | (a << 3) | m
}

pub static luaP_opmodes: [u8; NUM_OPCODES] = [
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 3),
    opmode(0, 0, 0, 0, 1, 3),
    opmode(0, 0, 0, 0, 1, 2),
    opmode(0, 0, 0, 0, 1, 2),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 0, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 0, 0),
    opmode(0, 0, 0, 0, 0, 0),
    opmode(0, 0, 0, 0, 0, 0),
    opmode(0, 0, 0, 0, 0, 0),
    opmode(0, 0, 0, 0, 1, 1),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(1, 0, 0, 0, 0, 0),
    opmode(1, 0, 0, 0, 0, 0),
    opmode(1, 0, 0, 0, 0, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 0, 0),
    opmode(0, 0, 0, 0, 0, 0),
    opmode(0, 0, 0, 0, 0, 5),
    opmode(0, 0, 0, 1, 0, 0),
    opmode(0, 0, 0, 1, 0, 0),
    opmode(0, 0, 0, 1, 0, 0),
    opmode(0, 0, 0, 1, 0, 0),
    opmode(0, 0, 0, 1, 0, 0),
    opmode(0, 0, 0, 1, 0, 0),
    opmode(0, 0, 0, 1, 0, 0),
    opmode(0, 0, 0, 1, 0, 0),
    opmode(0, 0, 0, 1, 0, 0),
    opmode(0, 0, 0, 1, 0, 0),
    opmode(0, 0, 0, 1, 1, 0),
    opmode(0, 1, 1, 0, 1, 0),
    opmode(0, 1, 1, 0, 1, 0),
    opmode(0, 0, 1, 0, 0, 0),
    opmode(0, 0, 0, 0, 0, 0),
    opmode(0, 0, 0, 0, 0, 0),
    opmode(0, 0, 0, 0, 1, 2),
    opmode(0, 0, 0, 0, 1, 2),
    opmode(0, 0, 0, 0, 0, 2),
    opmode(0, 0, 0, 0, 0, 0),
    opmode(0, 0, 0, 0, 1, 2),
    opmode(0, 0, 1, 0, 0, 1),
    opmode(0, 0, 0, 0, 1, 2),
    opmode(0, 1, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 1, 0),
    opmode(0, 0, 0, 0, 0, 2),
    opmode(0, 0, 1, 0, 1, 0),
    opmode(0, 0, 0, 0, 0, 4),
];

#[inline]
const fn mask1(n: u32, p: u32) -> Instruction {
    ((!((!0u32) << n)) << p) as Instruction
}

#[inline]
const fn get_opcode(i: Instruction) -> usize {
    ((i >> POS_OP) & mask1(7, 0)) as usize
}

#[inline]
const fn get_arg(i: Instruction, pos: u32, size: u32) -> c_int {
    ((i >> pos) & mask1(size, 0)) as c_int
}

#[inline]
const fn get_arg_b(i: Instruction) -> c_int {
    get_arg(i, POS_B, SIZE_B)
}

#[inline]
const fn get_arg_vb(i: Instruction) -> c_int {
    get_arg(i, POS_VB, SIZE_VB)
}

#[inline]
const fn get_arg_c(i: Instruction) -> c_int {
    get_arg(i, POS_C, SIZE_C)
}

#[inline]
const fn test_ot_mode(op: usize) -> bool {
    (luaP_opmodes[op] & (1 << 6)) != 0
}

#[inline]
const fn test_it_mode(op: usize) -> bool {
    (luaP_opmodes[op] & (1 << 5)) != 0
}

pub(crate) fn luaP_isOT(i: Instruction) -> c_int {
    let op = get_opcode(i);
    if op == OP_TAILCALL as usize {
        1
    } else {
        c_int::from(test_ot_mode(op) && get_arg_c(i) == 0)
    }
}

pub unsafe fn luaP_isIT(i: Instruction) -> c_int {
    let op = get_opcode(i);
    if op == OP_SETLIST as usize {
        c_int::from(test_it_mode(op) && get_arg_vb(i) == 0)
    } else {
        c_int::from(test_it_mode(op) && get_arg_b(i) == 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[inline]
    const fn create_abck(op: u32, a: u32, b: u32, c: u32, k: u32) -> Instruction {
        (op << POS_OP) | (a << POS_A) | (b << POS_B) | (c << POS_C) | (k << POS_K)
    }

    #[inline]
    const fn create_vabck(op: u32, a: u32, b: u32, c: u32, k: u32) -> Instruction {
        const SIZE_VC: u32 = 10;
        const POS_VC: u32 = POS_VB + SIZE_VB;
        (op << POS_OP) | (a << POS_A) | (b << POS_VB) | (c << POS_VC) | (k << POS_K)
    }

    #[test]
    fn opmode_table_has_expected_shape() {
        assert_eq!(luaP_opmodes.len(), NUM_OPCODES);
        assert_eq!(luaP_opmodes[OP_MOVE as usize] & 0b111, 0);
        assert_ne!(luaP_opmodes[OP_TAILCALL as usize] & (1 << 6), 0);
        assert_ne!(luaP_opmodes[OP_SETLIST as usize] & (1 << 5), 0);
    }

    #[test]
    fn detects_ot_and_it_instructions() {
        let call_multi = create_abck(68, 0, 0, 0, 0);
        let tailcall = create_abck(OP_TAILCALL as u32, 0, 1, 2, 0);
        let setlist_multi = create_vabck(OP_SETLIST as u32, 0, 0, 0, 0);
        let move_inst = create_abck(OP_MOVE as u32, 0, 1, 2, 0);

        assert_eq!(luaP_isOT(call_multi), 1);
        assert_eq!(luaP_isOT(tailcall), 1);
        assert_eq!(luaP_isOT(move_inst), 0);

        assert_eq!(unsafe { luaP_isIT(call_multi) }, 1);
        assert_eq!(unsafe { luaP_isIT(setlist_multi) }, 1);
        assert_eq!(unsafe { luaP_isIT(move_inst) }, 0);
    }
}
