#![feature(box_as_ptr)]
pub struct SSAstmt;
pub struct ESSAstmt;

#[used]
static _SSAstmt: Option<SSAstmt> = None;
static _ESSAstmt: Option<ESSAstmt> = None;
use std::slice;

fn main() {
    unsafe {
        let mut data = Box::new([10_u8, 20, 30]);
        let mut ptr = Box::as_mut_ptr(&mut data) as *const u8;
        let len = 3;

        let s1_slice: &[u8] = slice::from_raw_parts(ptr, len);
        let mut i: usize = 0;
        while i < len {
            let val1 = s1_slice[i];
            ptr = ptr.add(1);
            let val2 = *ptr;
            i += 1;
        }
    }
}
