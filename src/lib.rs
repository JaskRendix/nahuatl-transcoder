pub mod abugida;
pub mod atomic;
pub mod constants;
pub mod orthography;
pub mod structure;
pub mod trager;

use abugida::atomic_to_abugida;
use atomic::to_atomic;
use structure::structure;

pub fn convert(input: &str) -> String {
    let atomic = to_atomic(input);

    let raw_abugida = atomic_to_abugida(&atomic);

    structure(&raw_abugida)
}
