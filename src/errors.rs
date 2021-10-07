// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! A list of error types which are produced during an execution of the protocol
use displaydoc::Display;

#[cfg(feature = "std")]
use std::error::Error;

/// Represents an error in the manipulation of internal cryptographic data
#[derive(Debug, Display)]
pub enum InternalPakeError {
    /// Deserializing from a byte sequence failed
    InvalidByteSequence,
    /// Invalid length for {name}: expected {len}, but is actually {actual_len}.
    SizeError {
        /// name
        name: &'static str,
        /// length
        len: usize,
        /// actual
        actual_len: usize,
    },
    /// Could not decompress point.
    PointError,
    /// Key belongs to a small subgroup!
    SubGroupError,
    /// hashing to a key failed
    HashingFailure,
    /// Computing the hash-to-curve function failed
    HashToCurveError,
    /// Computing HKDF failed while deriving subkeys
    HkdfError,
    /// Computing HMAC failed while supplying a secret key
    HmacError,
    /// Computing the slow hashing function failed
    SlowHashError,
    /// Constructing the envelope seal failed.
    SealError,
    /// Opening the envelope seal failed.
    SealOpenError,
    /// HMAC check in seal open failed.
    SealOpenHmacError,
    /**
     * This error occurs when the envelope cannot be constructed properly
     * based on the credentials that were specified to be required.
     */
    InvalidEnvelopeStructureError,
    /**
     * This error occurs when attempting to open an envelope of the wrong
     * type (base mode, custom identifier)
     */
    IncompatibleEnvelopeModeError,
    /**
     * This error occurs when the envelope is opened and deserialization
     * fails
     */
    UnexpectedEnvelopeContentsError,
}

#[cfg(feature = "std")]
impl Error for InternalPakeError {}

/// Represents an error in password checking
#[derive(Debug, Display)]
pub enum PakeError {
    /// Internal error during PRF verification: {0}
    CryptoError(InternalPakeError),
    /// Incomplete set of keys passed into finish() function
    IncompleteKeysError,
    /// The provided server public key doesn't match the sealed one
    IncompatibleServerStaticPublicKeyError,
    /// Error in key exchange protocol when attempting to validate MACs
    KeyExchangeMacValidationError,
    /// Error in validating credentials
    InvalidLoginError,
    /// Error with serializing / deserializing protocol messages
    SerializationError,
    /// Identity group element was encountered during deserialization, which is invalid
    IdentityGroupElementError,
}

// This is meant to express future(ly) non-trivial ways of converting the
// internal error into a PakeError
impl From<InternalPakeError> for PakeError {
    fn from(e: InternalPakeError) -> PakeError {
        PakeError::CryptoError(e)
    }
}

#[cfg(feature = "std")]
impl Error for PakeError {}

/// Represents an error in protocol handling
#[derive(Debug, Display)]
pub enum ProtocolError {
    /// Internal error during password verification: {0}
    VerificationError(PakeError),
    /// Server response cannot be handled.
    ServerError,
    /**
     * This error occurs when the server specifies an envelope credentials
     * format that is invalid
     */
    ServerInvalidEnvelopeCredentialsFormatError,
    /// Client request cannot be handled.
    ClientError,
    /**
     * This error occurs when the client detects that the server has
     * reflected the OPRF value (beta == alpha)
     */
    ReflectedValueError,
}

#[cfg(feature = "std")]
impl Error for ProtocolError {}

// This is meant to express future(ly) non-trivial ways of converting the
// Pake error into a ProtocolError
impl From<PakeError> for ProtocolError {
    fn from(e: PakeError) -> ProtocolError {
        ProtocolError::VerificationError(e)
    }
}

// This is meant to express future(ly) non-trivial ways of converting the
// internal error into a ProtocolError
impl From<InternalPakeError> for ProtocolError {
    fn from(e: InternalPakeError) -> ProtocolError {
        ProtocolError::VerificationError(e.into())
    }
}

// See https://github.com/rust-lang/rust/issues/64715 and remove this when
// merged, and https://github.com/dtolnay/thiserror/issues/62 for why this
// comes up in our doc tests.
impl From<::core::convert::Infallible> for ProtocolError {
    fn from(_: ::core::convert::Infallible) -> Self {
        unreachable!()
    }
}

pub(crate) mod utils {
    use super::*;

    pub fn check_slice_size<'a>(
        slice: &'a [u8],
        expected_len: usize,
        arg_name: &'static str,
    ) -> Result<&'a [u8], InternalPakeError> {
        if slice.len() != expected_len {
            return Err(InternalPakeError::SizeError {
                name: arg_name,
                len: expected_len,
                actual_len: slice.len(),
            });
        }
        Ok(slice)
    }

    pub fn check_slice_size_atleast<'a>(
        slice: &'a [u8],
        expected_len: usize,
        arg_name: &'static str,
    ) -> Result<&'a [u8], InternalPakeError> {
        if slice.len() < expected_len {
            return Err(InternalPakeError::SizeError {
                name: arg_name,
                len: expected_len,
                actual_len: slice.len(),
            });
        }
        Ok(slice)
    }
}
