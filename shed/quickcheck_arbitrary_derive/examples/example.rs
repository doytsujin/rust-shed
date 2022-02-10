/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

#![allow(dead_code)]

use quickcheck_arbitrary_derive::Arbitrary;

#[derive(Arbitrary, Clone, Debug)]
struct StructFoo {
    bar: u8,
    baz: String,
}

#[derive(Arbitrary, Clone, Debug)]
struct UnitFoo;

#[derive(Arbitrary, Clone, Debug)]
struct TupleFoo(u8, String);

#[derive(Arbitrary, Clone, Debug)]
enum EnumFoo {
    Foo { foo: StructFoo, bar: Vec<u8> },
    Bar { hello: i64 },
    Baz(u8),
    Qux,
}

fn main() {
    use quickcheck::{Arbitrary, Gen};
    let mut gen = Gen::new(10);
    println!("{:#?}", StructFoo::arbitrary(&mut gen));
    println!("{:#?}", TupleFoo::arbitrary(&mut gen));
    println!("{:#?}", UnitFoo::arbitrary(&mut gen));
    println!("{:#?}", EnumFoo::arbitrary(&mut gen));
}
