fn main() {
    let a = 42;
    let b = a; // 复制 a
    let c = b; // 复制 b
    let d = c; // 复制 c
    let e = d + 1;

    let emd = e + 10;
    // let end = odd(emd);
    println!("{}", emd); // 这样 Rust 就不能优化掉变量了
}
// fn odd(emd: i32) -> i32 {
//     let a = 42;
//     let b = a;  // 复制 a
//     let c = b;  // 复制 b
//     let d = c;  // 复制 c
//     let e = d + 1;

//     let emd = e+10;
//     emd

// }
