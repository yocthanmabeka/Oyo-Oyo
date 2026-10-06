// Un module qui boucle sans fin, exprès : la preuve que le moteur l'arrête (ADR-011, partie C).
#![no_std]

#[panic_handler]
fn panique(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[no_mangle]
pub extern "C" fn run(n: u32) -> u32 {
    let mut x = n;
    loop {
        x = core::hint::black_box(x.wrapping_add(1));
    }
}
