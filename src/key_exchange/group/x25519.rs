// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Key Exchange group implementation for x25519

use super::KeGroup;
use crate::errors::InternalError;
use curve25519_dalek::{constants::X25519_BASEPOINT, montgomery::MontgomeryPoint, scalar::Scalar};
use generic_array::{typenum::U32, GenericArray};
use rand::{CryptoRng, RngCore};

/// The implementation of such a subgroup for Ristretto
impl KeGroup for MontgomeryPoint {
    type PkLen = U32;
    type SkLen = U32;

    fn from_pk_slice(element_bits: &GenericArray<u8, Self::PkLen>) -> Result<Self, InternalError> {
        Ok(Self(*element_bits.as_ref()))
    }

    fn random_sk<R: RngCore + CryptoRng>(rng: &mut R) -> GenericArray<u8, Self::SkLen> {
        loop {
            let scalar = {
                #[cfg(not(test))]
                {
                    let mut scalar_bytes = [0u8; 64];
                    rng.fill_bytes(&mut scalar_bytes);
                    Scalar::from_bytes_mod_order_wide(&scalar_bytes)
                }

                // Tests need an exact conversion from bytes to scalar, sampling only 32 bytes from rng
                #[cfg(test)]
                {
                    let mut scalar_bytes = [0u8; 32];
                    rng.fill_bytes(&mut scalar_bytes);
                    Scalar::from_bytes_mod_order(scalar_bytes)
                }
            };

            if scalar != Scalar::zero() {
                break GenericArray::clone_from_slice(&scalar.to_bytes());
            }
        }
    }

    fn public_key(sk: &GenericArray<u8, Self::SkLen>) -> Self {
        X25519_BASEPOINT * Scalar::from_bits(*sk.as_ref())
    }

    fn to_arr(&self) -> GenericArray<u8, Self::PkLen> {
        self.to_bytes().into()
    }

    fn diffie_hellman(&self, sk: &GenericArray<u8, Self::SkLen>) -> GenericArray<u8, Self::PkLen> {
        (self * Scalar::from_bits(*sk.as_ref())).to_arr()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ProtocolError;

    #[test]
    fn test_x25519() -> Result<(), ProtocolError> {
        use crate::{
            key_exchange::tripledh::TripleDH, slow_hash::NoOpHash, CipherSuite, ClientLogin,
            ClientLoginFinishParameters, ClientLoginFinishResult, ClientLoginStartResult,
            ClientRegistration, ClientRegistrationFinishParameters, ClientRegistrationFinishResult,
            ClientRegistrationStartResult, ServerLogin, ServerLoginStartParameters,
            ServerLoginStartResult, ServerRegistration, ServerSetup,
        };
        use curve25519_dalek::ristretto::RistrettoPoint;
        use rand::rngs::OsRng;

        struct X25519Sha512NoSlowHash;
        impl CipherSuite for X25519Sha512NoSlowHash {
            type OprfGroup = RistrettoPoint;
            type KeGroup = MontgomeryPoint;
            type KeyExchange = TripleDH;
            type Hash = sha2::Sha512;
            type SlowHash = NoOpHash;
        }

        const PASSWORD: &[u8] = b"1234";

        let server_setup = ServerSetup::<X25519Sha512NoSlowHash>::new(&mut OsRng)?;

        let ClientRegistrationStartResult {
            message,
            state: client,
        } = ClientRegistration::start(&mut OsRng, PASSWORD)?;
        let message = ServerRegistration::start(&server_setup, message, &[])?.message;
        let ClientRegistrationFinishResult {
            message,
            export_key: register_export_key,
            ..
        } = client.finish(
            &mut OsRng,
            message,
            ClientRegistrationFinishParameters::default(),
        )?;
        let server_registration = ServerRegistration::finish(message);

        let ClientLoginStartResult {
            message,
            state: client,
        } = ClientLogin::start(&mut OsRng, PASSWORD)?;
        let ServerLoginStartResult {
            message,
            state: server,
            ..
        } = ServerLogin::start(
            &mut OsRng,
            &server_setup,
            Some(server_registration),
            message,
            &[],
            ServerLoginStartParameters::default(),
        )?;
        let ClientLoginFinishResult {
            message,
            session_key: client_session_key,
            export_key: login_export_key,
            ..
        } = client.finish(message, ClientLoginFinishParameters::default())?;
        let server_session_key = server.finish(message)?.session_key;

        assert_eq!(register_export_key, login_export_key);
        assert_eq!(client_session_key, server_session_key);

        let ClientLoginStartResult {
            message,
            state: client,
        } = ClientLogin::start(&mut OsRng, PASSWORD)?;
        let ServerLoginStartResult { message, .. } = ServerLogin::start(
            &mut OsRng,
            &server_setup,
            None,
            message,
            &[],
            ServerLoginStartParameters::default(),
        )?;

        assert!(matches!(
            client.finish(message, ClientLoginFinishParameters::default()),
            Err(ProtocolError::InvalidLoginError)
        ));

        Ok(())
    }
}
