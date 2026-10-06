// Un module qui calcule : la somme des nombres de 1 à n. Il ne reçoit qu'un nombre et ne rend
// qu'un nombre ; il n'a accès à rien d'autre (ni réseau, ni page, ni heure).
#![no_std]

#[panic_handler]
fn panique(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[no_mangle]
pub extern "C" fn run(n: u32) -> u32 {
    let mut somme: u32 = 0;
    for i in 1..=n {
        somme = somme.wrapping_add(i);
    }
    somme
}
