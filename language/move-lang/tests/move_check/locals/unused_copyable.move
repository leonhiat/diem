module M {
    struct S {}

    // this produces unused parameter warnings for i and s, but not unused resource warnings
    fun t0(i: u64, s: S) {
    }

    fun t1() {
        let s = S{};
    }

    fun t2() {
        // prefixing an unused non-resource with _ suppresses the warning
        let _s = S{};
    }
}
