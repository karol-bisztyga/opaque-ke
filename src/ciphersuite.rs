// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Defines the CipherSuite trait to specify the underlying primitives for OPAQUE

use crate::key_exchange::group::KeGroup;
use crate::{hash::Hash, key_exchange::traits::KeyExchange, slow_hash::SlowHash};
use voprf::group::Group as OprfGroup;

/// Configures the underlying primitives used in OPAQUE
/// * `OprfGroup`: a finite cyclic group along with a point representation, along
///   with an extension trait PasswordToCurve that allows some customization on
///   how to hash a password to a curve point. See `group::Group`.
/// * `KeGroup`: A `Group` used for the `KeyExchange`.
/// * `KeyExchange`: The key exchange protocol to use in the login step
/// * `Hash`: The main hashing function to use
/// * `SlowHash`: A slow hashing function, typically used for password hashing
pub trait CipherSuite {
    /// A finite cyclic group along with a point representation along with
    /// an extension trait PasswordToCurve that allows some customization on
    /// how to hash a password to a curve point. See `group::Group`.
    type OprfGroup: OprfGroup;
    /// A `Group` used for the `KeyExchange`.
    type KeGroup: KeGroup;
    /// A key exchange protocol
    type KeyExchange: KeyExchange<Self::Hash, Self::KeGroup>;
    /// The main hash function use (for HKDF computations and hashing transcripts)
    type Hash: Hash;
    /// A slow hashing function, typically used for password hashing
    type SlowHash: SlowHash<Self::Hash>;
}
