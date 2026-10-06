// Un module qui réclame trop de mémoire, exprès : il demande 64 Mo d'un coup. Le moteur ne lui en
// a donné qu'un plafond (memory:) ; la demande échoue, et le module s'arrête.
#![no_std]

#[panic_handler]
fn panique(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[no_mangle]
pub extern "C" fn run(n: u32) -> u32 {
    // 1024 pages de 64 Ko : 64 Mo. Au-delà du plafond, la mémoire ne grandit pas (-1).
    let avant = core::arch::wasm32::memory_grow(0, 1024);
    if avant == usize::MAX {
        core::arch::wasm32::unreachable();
    }
    n
}
