// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! A list of error types which are produced during an execution of the protocol
use core::convert::Infallible;
use core::fmt::Debug;
#[cfg(feature = "std")]
use std::error::Error;

use displaydoc::Display;

/// Represents an error in the manipulation of internal cryptographic data
#[derive(Clone, Display, Eq, Hash, PartialEq)]
pub enum InternalError<T = Infallible> {
    /// Custom [`SecretKey`](crate::keypair::SecretKey) error type
    Custom(T),
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
    /// Computing the hash-to-curve function failed
    HashToCurveError,
    /// Computing HKDF failed while deriving subkeys
    HkdfError,
    /// Computing HMAC failed while supplying a secret key
    HmacError,
    /// Computing the slow hashing function failed
    SlowHashError,
    /** This error occurs when the envelope seal open hmac check fails
    HMAC check in seal open failed. */
    SealOpenHmacError,
    /** This error occurs when attempting to open an envelope of the wrong
    type (base mode, custom identifier) */
    IncompatibleEnvelopeModeError,
    /// This error occurs when the inner envelope is malformed
    InvalidInnerEnvelopeError,
    /// Error from the OPRF evaluation
    OprfError(voprf::errors::InternalError),
    /// Error encountered when attempting to produce a keypair
    InvalidKeypairError,
}

impl<T: Debug> Debug for InternalError<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Custom(custom) => f.debug_tuple("InvalidByteSequence").field(custom).finish(),
            Self::InvalidByteSequence => f.debug_tuple("InvalidByteSequence").finish(),
            Self::SizeError {
                name,
                len,
                actual_len,
            } => f
                .debug_struct("SizeError")
                .field("name", name)
                .field("len", len)
                .field("actual_len", actual_len)
                .finish(),
            Self::PointError => f.debug_tuple("PointError").finish(),
            Self::HashToCurveError => f.debug_tuple("HashToCurveError").finish(),
            Self::HkdfError => f.debug_tuple("HkdfError").finish(),
            Self::HmacError => f.debug_tuple("HmacError").finish(),
            Self::SlowHashError => f.debug_tuple("SlowHashError").finish(),
            Self::SealOpenHmacError => f.debug_tuple("SealOpenHmacError").finish(),
            Self::IncompatibleEnvelopeModeError => {
                f.debug_tuple("IncompatibleEnvelopeModeError").finish()
            }
            Self::InvalidInnerEnvelopeError => f.debug_tuple("InvalidInnerEnvelopeError").finish(),
            Self::OprfError(error) => f.debug_tuple("OprfError").field(error).finish(),
            Self::InvalidKeypairError => f.debug_tuple("InvalidKeypairError").finish(),
        }
    }
}

#[cfg(feature = "std")]
impl<T: Error> Error for InternalError<T> {}

impl InternalError {
    /// Convert `InternalError<Infallible>` into `InternalError<T>
    pub fn into_custom<T>(self) -> InternalError<T> {
        match self {
            Self::Custom(_) => unreachable!(),
            Self::InvalidByteSequence => InternalError::InvalidByteSequence,
            Self::SizeError {
                name,
                len,
                actual_len,
            } => InternalError::SizeError {
                name,
                len,
                actual_len,
            },
            Self::PointError => InternalError::PointError,
            Self::HashToCurveError => InternalError::HashToCurveError,
            Self::HkdfError => InternalError::HkdfError,
            Self::HmacError => InternalError::HmacError,
            Self::SlowHashError => InternalError::SlowHashError,
            Self::SealOpenHmacError => InternalError::SealOpenHmacError,
            Self::IncompatibleEnvelopeModeError => InternalError::IncompatibleEnvelopeModeError,
            Self::InvalidInnerEnvelopeError => InternalError::InvalidInnerEnvelopeError,
            Self::OprfError(error) => InternalError::OprfError(error),
            Self::InvalidKeypairError => InternalError::InvalidKeypairError,
        }
    }
}

impl From<voprf::errors::InternalError> for InternalError {
    fn from(voprf_error: voprf::errors::InternalError) -> Self {
        Self::OprfError(voprf_error)
    }
}

impl From<voprf::errors::InternalError> for ProtocolError {
    fn from(voprf_error: voprf::errors::InternalError) -> Self {
        Self::LibraryError(InternalError::OprfError(voprf_error))
    }
}

/// Represents an error in protocol handling
#[derive(Clone, Display, Eq, Hash, PartialEq)]
pub enum ProtocolError<T = Infallible> {
    /// Internal error encountered
    LibraryError(InternalError<T>),
    /// Error in validating credentials
    InvalidLoginError,
    /// Error with serializing / deserializing protocol messages
    SerializationError,
    /** This error occurs when the client detects that the server has
    reflected the OPRF value (beta == alpha) */
    ReflectedValueError,
    /// Identity group element was encountered during deserialization, which is invalid
    IdentityGroupElementError,
}

impl<T: Debug> Debug for ProtocolError<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::LibraryError(pake_error) => {
                f.debug_tuple("LibraryError").field(pake_error).finish()
            }
            Self::InvalidLoginError => f.debug_tuple("InvalidLoginError").finish(),
            Self::SerializationError => f.debug_tuple("SerializationError").finish(),
            Self::ReflectedValueError => f.debug_tuple("ReflectedValueError").finish(),
            Self::IdentityGroupElementError => f.debug_tuple("IdentityGroupElementError").finish(),
        }
    }
}

#[cfg(feature = "std")]
impl<T: Error> Error for ProtocolError<T> {}

// This is meant to express future(ly) non-trivial ways of converting the
// internal error into a ProtocolError
impl<T> From<InternalError<T>> for ProtocolError<T> {
    fn from(e: InternalError<T>) -> ProtocolError<T> {
        Self::LibraryError(e)
    }
}

// See https://github.com/rust-lang/rust/issues/64715 and remove this when
// merged, and https://github.com/dtolnay/thiserror/issues/62 for why this
// comes up in our doc tests.
impl<T> From<::core::convert::Infallible> for ProtocolError<T> {
    fn from(_: ::core::convert::Infallible) -> Self {
        unreachable!()
    }
}

impl ProtocolError {
    /// Convert `ProtocolError<Infallible>` into `ProtocolError<T>
    pub fn into_custom<T>(self) -> ProtocolError<T> {
        match self {
            Self::LibraryError(internal_error) => {
                ProtocolError::LibraryError(internal_error.into_custom())
            }
            Self::InvalidLoginError => ProtocolError::InvalidLoginError,
            Self::SerializationError => ProtocolError::SerializationError,
            Self::ReflectedValueError => ProtocolError::ReflectedValueError,
            Self::IdentityGroupElementError => ProtocolError::IdentityGroupElementError,
        }
    }
}

pub(crate) mod utils {
    use super::*;

    pub fn check_slice_size<'a, T>(
        slice: &'a [u8],
        expected_len: usize,
        arg_name: &'static str,
    ) -> Result<&'a [u8], InternalError<T>> {
        if slice.len() != expected_len {
            return Err(InternalError::SizeError {
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
    ) -> Result<&'a [u8], InternalError> {
        if slice.len() < expected_len {
            return Err(InternalError::SizeError {
                name: arg_name,
                len: expected_len,
                actual_len: slice.len(),
            });
        }
        Ok(slice)
    }
}
