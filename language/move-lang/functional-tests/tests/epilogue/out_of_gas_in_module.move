module Swapper {
    use 0x1::Vector;
    public fun swap_it_up(vec_len: u64) {
        let v = Vector::empty();

        let i = 0;
        while (i < vec_len) {
          Vector::push_back(&mut v, i);
          i = i + 1;
        };

        i = 0;

        while (i < vec_len / 2) {
            Vector::swap(&mut v, i, vec_len - i - 1);
            i = i + 1;
        };
    }
}

//! new-transaction
//! max-gas: 620
script {
use {{default}}::Swapper;
fun main() {
    Swapper::swap_it_up(10000)
}
}
// check: "EXECUTION_FAILURE { status_code: OUT_OF_GAS, location: A4A46D1B1421502568A4A6AC326D7250::Swapper,"
// check: "gas_used: 620,"
// check: "Keep(OUT_OF_GAS)"
