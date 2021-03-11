module Test {
    struct R has drop {
        x: u64,
        s: S,
    }

    struct S has drop {
        y: u64
    }

    spec struct R {
        invariant x > s.y;
    }

    spec struct S {
        invariant y > 0;
    }

    public fun test_param(_simple_R: R, _ref_R: &R, _simple_S: S, _mut_R: &mut R) {}
}
