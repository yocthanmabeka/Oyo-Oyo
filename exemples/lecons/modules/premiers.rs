// Un module venu d'ailleurs (leçon 141, ADR-118) : combien de nombres premiers jusqu'à n. Il ne
// reçoit qu'un nombre et ne rend qu'un nombre ; il n'a accès à rien d'autre (ni réseau, ni page,
// ni heure). La page le lance avec son empreinte : si le fichier change, il n'est pas lancé.
#![no_std]

#[panic_handler]
fn panique(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[no_mangle]
pub extern "C" fn run(n: u32) -> u32 {
    let mut premiers = 0;
    let mut k = 2;
    while k <= n {
        // k est premier si aucun nombre de 2 à sa racine ne le divise.
        let mut diviseur = 2;
        let mut premier = true;
        while diviseur <= k / diviseur {
            if k % diviseur == 0 {
                premier = false;
                break;
            }
            diviseur += 1;
        }
        if premier {
            premiers += 1;
        }
        k += 1;
    }
    premiers
}
