/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use fbinit::FacebookInit;

#[cfg(fbcode_build)]
#[fbinit::test]
fn test_was_performed_success() {
    assert!(fbinit::was_performed())
}

#[cfg(fbcode_build)]
#[test]
fn test_was_performed_false() {
    assert!(!fbinit::was_performed())
}

#[cfg(not(fbcode_build))]
#[fbinit::test]
fn test_was_performed_oss_false_with_proof() {
    assert!(!fbinit::was_performed())
}

#[cfg(not(fbcode_build))]
#[test]
fn test_was_performed_false_regardless() {
    assert!(!fbinit::was_performed())
}

#[fbinit::test]
fn test_without_proof() {}

#[fbinit::test]
fn test_with_proof(fb: FacebookInit) {
    println!("Got fb: {:?}", fb);
}

/// This can work only on fbcode builds as only then the proof can be asserted
#[cfg(fbcode_build)]
#[fbinit::test]
fn test_expect_init() {
    fbinit::expect_init();
}

/// Also works with disable_fatal_signals set
#[cfg(fbcode_build)]
#[fbinit::test(disable_fatal_signals = sigterm_only)]
fn test_expect_init_with_disable_signals() {
    fbinit::expect_init();
}

/// On non-fbcode builds asserting the proof will always panic, even in fbinit::test
#[cfg(not(fbcode_build))]
#[fbinit::test]
#[should_panic]
fn test_expect_init() {
    fbinit::expect_init();
}

#[test]
#[should_panic]
fn test_expect_init_panics() {
    fbinit::expect_init();
}

/// This can work only on fbcode builds as only then the proof can be asserted
#[cfg(fbcode_build)]
#[fbinit::test]
fn test_main_expect_init() {
    #[fbinit::main]
    fn main() {
        fbinit::expect_init();
    }

    main();
}

/// On non-fbcode builds asserting the proof will always panic, even in fbinit::test
#[cfg(not(fbcode_build))]
#[fbinit::test]
#[should_panic]
fn test_main_expect_init() {
    #[fbinit::main]
    fn main() {
        fbinit::expect_init();
    }

    main();
}

#[fbinit::test]
async fn test_async_without_proof() {
    async fn helper() {}

    helper().await;
}

#[fbinit::test]
async fn test_async_with_proof(fb: FacebookInit) {
    async fn helper(_fb: FacebookInit) {}

    helper(fb).await;
}

#[test]
fn test_main_without_proof() {
    #[fbinit::main]
    fn main() {}

    main();
}

#[test]
fn test_main_with_proof() {
    #[fbinit::main]
    fn main(fb: FacebookInit) {
        println!("Got fb: {:?}", fb);
    }

    main();
}

mod submodule {
    #[fbinit::main]
    fn main() {}

    #[test]
    #[should_panic(expected = "fbinit must be performed in the crate root on the main function")]
    fn test_in_submodule() {
        main();
    }
}

#[test]
fn test_main_with_disable_signals_sigterm_only() {
    #[fbinit::main(disable_fatal_signals = sigterm_only)]
    fn main() {}

    main();
}

#[test]
fn test_main_with_disable_signals_none() {
    #[fbinit::main(disable_fatal_signals = none)]
    fn main() {}

    main();
}

#[test]
fn test_main_with_disable_signals_all() {
    #[fbinit::main(disable_fatal_signals = all)]
    fn main() {}

    main();
}

#[test]
fn test_main_with_disable_signals_default() {
    #[fbinit::main(disable_fatal_signals = default)]
    fn main() {}

    main();
}
