use fbool::entanglement::Entropy;
use fbool::fvalue::FValue;

/// Handle the debug command
pub async fn handle_debug() {
    let n = 3;
    let formula = FValue::primality(n);

    for i in 0..n {
        for j in i..n {
            println!("{}", formula.permutate_var(i, j).entropy());
        }
    }
}
