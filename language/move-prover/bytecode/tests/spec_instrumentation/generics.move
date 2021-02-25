// flag: --v2
module Generics {

    spec module {
        pragma verify = true;
    }

    struct R<T> has key { x: T }

    fun remove<T: store>(a: address): R<T> acquires R {
        move_from<R<T>>(a)
    }
    spec fun remove {
        pragma opaque;
        include Remove<T>;
    }
    spec schema Remove<T> {
        a: address;
        modifies global<R<T>>(a);
        ensures !exists<R<T>>(a);
    }

    fun remove_u64(a: address): R<u64> acquires R {
        remove<u64>(a)
    }
    spec fun remove_u64 {
        include Remove<u64>;
    }
}
