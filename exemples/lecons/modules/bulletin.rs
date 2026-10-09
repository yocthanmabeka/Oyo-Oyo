// Un module du second contrat (ADR-077) : il reçoit la liste des notes, chacune avec sa matière,
// et rend la moyenne, la meilleure matière et le nombre de notes. Il ne reçoit qu'un texte JSON
// et ne rend qu'un texte JSON ; il n'a accès à rien d'autre (ni réseau, ni page, ni heure).
//
// Ce qu'il reçoit : {"notes":[{"matiere":"Maths","note":"15.5"},…]}
// Ce qu'il rend   : {"moyenne":13.5,"meilleure":"Maths","nombre":3}
#![no_std]

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

const INPUT_MAX: usize = 16_384;
static mut INPUT: [u8; INPUT_MAX] = [0; INPUT_MAX];
static mut OUTPUT: [u8; 1024] = [0; 1024];

/// Où le moteur écrit ce que le module reçoit ; 0 s'il n'y a pas la place.
#[no_mangle]
pub extern "C" fn alloc(size: u32) -> u32 {
    if size as usize > INPUT_MAX {
        return 0;
    }
    core::ptr::addr_of_mut!(INPUT) as u32
}

/// Lit les notes, et rend l'adresse et la taille de la réponse en un seul nombre.
#[no_mangle]
pub extern "C" fn run(at: u32, size: u32) -> u64 {
    let input = unsafe { core::slice::from_raw_parts(at as *const u8, size as usize) };
    let output = unsafe { &mut *core::ptr::addr_of_mut!(OUTPUT) };
    let address = output.as_ptr() as u64;
    let (mut total, mut count, mut best) = (0u64, 0u64, 0u64);
    let mut best_subject: &[u8] = &[];
    // La liste commence au premier crochet ; chaque élément est un objet : on lit ses deux champs.
    let mut i = find(input, 0, b'[').map_or(input.len(), |at| at + 1);
    while let Some(start) = find(input, i, b'{') {
        let Some(end) = object_end(input, start) else { break };
        let object = &input[start..=end];
        if let (Some(subject), Some(mark)) = (field(object, b"matiere"), field(object, b"note")) {
            let tenths = tenths(mark);
            if count == 0 || tenths > best {
                best = tenths;
                best_subject = subject;
            }
            total += tenths;
            count += 1;
        }
        i = end + 1;
    }
    // La moyenne en dixièmes, arrondie.
    let average = if count == 0 { 0 } else { (total + count / 2) / count };
    let mut out = Writer { buffer: output, length: 0 };
    out.text(b"{\"moyenne\":");
    out.number(average / 10);
    out.text(b".");
    out.number(average % 10);
    out.text(b",\"meilleure\":\"");
    // Le texte arrive déjà écrit en JSON (ses guillemets échappés) : il repart tel quel.
    out.text(best_subject);
    out.text(b"\",\"nombre\":");
    out.number(count);
    out.text(b"}");
    (address << 32) | out.length as u64
}

/// La première position de `byte` à partir de `from`, hors des textes.
fn find(input: &[u8], from: usize, byte: u8) -> Option<usize> {
    let mut i = from;
    while i < input.len() {
        match input[i] {
            b'"' => i = string_end(input, i)?,
            b if b == byte => return Some(i),
            _ => {}
        }
        i += 1;
    }
    None
}

/// La fin de l'objet qui commence à `start` (pas d'objet dans l'objet ici).
fn object_end(input: &[u8], start: usize) -> Option<usize> {
    find(input, start + 1, b'}')
}

/// La position du guillemet qui ferme le texte ouvert en `start`.
fn string_end(input: &[u8], start: usize) -> Option<usize> {
    let mut i = start + 1;
    while i < input.len() {
        match input[i] {
            b'\\' => i += 2,
            b'"' => return Some(i),
            _ => i += 1,
        }
    }
    None
}

/// La valeur (un texte) du champ `name` d'un objet, telle qu'écrite en JSON, sans ses guillemets.
fn field<'a>(object: &'a [u8], name: &[u8]) -> Option<&'a [u8]> {
    let mut i = 0;
    while let Some(key_start) = object[i..].iter().position(|&b| b == b'"').map(|at| i + at) {
        let key_end = string_end(object, key_start)?;
        let key = &object[key_start + 1..key_end];
        let mut j = key_end + 1;
        while j < object.len() && object[j] == b' ' {
            j += 1;
        }
        if object.get(j) == Some(&b':') {
            j += 1;
            while j < object.len() && object[j] == b' ' {
                j += 1;
            }
            if object.get(j) != Some(&b'"') {
                return None;
            }
            let value_end = string_end(object, j)?;
            if key == name {
                return Some(&object[j + 1..value_end]);
            }
            i = value_end + 1;
        } else {
            i = key_end + 1;
        }
    }
    None
}

/// « 15.5 » en dixièmes : 155. Les chiffres en trop après la virgule sont laissés.
fn tenths(written: &[u8]) -> u64 {
    let (mut units, mut tenth, mut after) = (0u64, 0u64, false);
    for &c in written {
        match c {
            b'0'..=b'9' if !after => units = units.saturating_mul(10).saturating_add((c - b'0') as u64),
            b'0'..=b'9' => {
                tenth = (c - b'0') as u64;
                break;
            }
            b'.' | b',' => after = true,
            _ => break,
        }
    }
    units.min(1_000_000) * 10 + tenth
}

/// Écrit la réponse dans sa mémoire, sans jamais déborder.
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

    fn number(&mut self, mut n: u64) {
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
