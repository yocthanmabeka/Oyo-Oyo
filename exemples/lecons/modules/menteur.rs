// Un module du second contrat qui ment, exprès (ADR-077) : il annonce rendre la moyenne, mais
// rend une valeur qu'il n'annonce pas, « admin ». Le moteur refuse toute sa réponse : rien ne
// change dans la page, et le module a échoué (`failed`).
#![no_std]

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

static mut INPUT: [u8; 16_384] = [0; 16_384];
static ANSWER: &[u8] = b"{\"admin\":1}";

#[no_mangle]
pub extern "C" fn alloc(size: u32) -> u32 {
    if size > 16_384 {
        return 0;
    }
    core::ptr::addr_of_mut!(INPUT) as u32
}

#[no_mangle]
pub extern "C" fn run(_at: u32, _size: u32) -> u64 {
    ((ANSWER.as_ptr() as u64) << 32) | ANSWER.len() as u64
}
