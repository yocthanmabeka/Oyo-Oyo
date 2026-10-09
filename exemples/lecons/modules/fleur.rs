// Un module qui dessine (ADR-088) : il reçoit un nombre de pétales et rend une liste de formes,
// comme n'importe quelle valeur. Il ne touche jamais au dessin du navigateur : le moteur vérifie
// chaque forme avant de la dessiner.
//
// Ce qu'il reçoit : {"petales":6}
// Ce qu'il rend   : {"fleur":[{"form":"line",…},{"form":"circle",…},…]}
#![no_std]

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

const INPUT_MAX: usize = 1024;
static mut INPUT: [u8; INPUT_MAX] = [0; INPUT_MAX];
static mut OUTPUT: [u8; 8192] = [0; 8192];

const PI: f64 = 3.141_592_653_589_793;

#[no_mangle]
pub extern "C" fn alloc(size: u32) -> u32 {
    if size as usize > INPUT_MAX {
        return 0;
    }
    core::ptr::addr_of_mut!(INPUT) as u32
}

#[no_mangle]
pub extern "C" fn run(at: u32, size: u32) -> u64 {
    let input = unsafe { core::slice::from_raw_parts(at as *const u8, size as usize) };
    let output = unsafe { &mut *core::ptr::addr_of_mut!(OUTPUT) };
    let address = output.as_ptr() as u64;
    let petals = number_after(input, b"\"petales\":").clamp(1, 24);
    let mut out = Writer { buffer: output, length: 0 };
    out.text(b"{\"fleur\":[");
    // La tige, puis les pétales en rond, puis le cœur, qui passe devant.
    out.text(b"{\"form\":\"line\",\"x1\":160,\"y1\":96,\"x2\":160,\"y2\":172,\"stroke\":\"#9be7a1\",\"thickness\":4}");
    for i in 0..petals {
        let angle = 2.0 * PI * i as f64 / petals as f64 - PI / 2.0;
        let x = 160.0 + 36.0 * cos(angle);
        let y = 80.0 + 36.0 * sin(angle);
        out.text(b",{\"form\":\"circle\",\"x\":");
        out.number(x);
        out.text(b",\"y\":");
        out.number(y);
        out.text(if i % 2 == 0 { b",\"r\":17,\"fill\":\"#ff8fa3\"}" } else { b",\"r\":17,\"fill\":\"#c3a6ff\"}" });
    }
    out.text(b",{\"form\":\"circle\",\"x\":160,\"y\":80,\"r\":15,\"fill\":\"#E9B44C\"}]}");
    (address << 32) | out.length as u64
}

/// Le nombre entier écrit après `key` ; 0 s'il n'y en a pas.
fn number_after(input: &[u8], key: &[u8]) -> u64 {
    let Some(start) = input.windows(key.len()).position(|w| w == key) else { return 0 };
    let mut n = 0u64;
    for &c in input[start + key.len()..].iter().skip_while(|c| **c == b' ') {
        if !c.is_ascii_digit() {
            break;
        }
        n = n.saturating_mul(10).saturating_add((c - b'0') as u64);
    }
    n
}

/// Le sinus, sans bibliothèque : l'angle ramené entre -π et π, puis une série de Taylor.
fn sin(mut x: f64) -> f64 {
    while x > PI {
        x -= 2.0 * PI;
    }
    while x < -PI {
        x += 2.0 * PI;
    }
    let x2 = x * x;
    x * (1.0 - x2 / 6.0 * (1.0 - x2 / 20.0 * (1.0 - x2 / 42.0 * (1.0 - x2 / 72.0 * (1.0 - x2 / 110.0 * (1.0 - x2 / 156.0))))))
}

fn cos(x: f64) -> f64 {
    sin(x + PI / 2.0)
}

struct Writer<'a> {
    buffer: &'a mut [u8],
    length: usize,
}

impl Writer<'_> {
    fn text(&mut self, bytes: &[u8]) {
        for &b in bytes {
            if self.length < self.buffer.len() {
                self.buffer[self.length] = b;
                self.length += 1;
            }
        }
    }

    /// Un nombre arrondi à l'unité, jamais négatif.
    fn number(&mut self, value: f64) {
        let mut n = if value <= 0.0 { 0 } else { (value + 0.5) as u64 };
        let mut digits = [0u8; 20];
        let mut i = digits.len();
        loop {
            i -= 1;
            digits[i] = b'0' + (n % 10) as u8;
            n /= 10;
            if n == 0 {
                break;
            }
        }
        self.text(&digits[i..]);
    }
}
