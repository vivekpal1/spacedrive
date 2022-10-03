use aead::{stream::EncryptorLE31, KeyInit, Payload};
use aes_gcm::Aes256Gcm;
use chacha20poly1305::XChaCha20Poly1305;
use secrecy::{ExposeSecret, Secret};

use crate::primitives::{Algorithm, Mode};

pub enum StreamEncryption {
	XChaCha20Poly1305(Box<EncryptorLE31<XChaCha20Poly1305>>),
	Aes256Gcm(Box<EncryptorLE31<Aes256Gcm>>),
}

impl StreamEncryption {
	pub fn init(key: Secret<[u8; 32]>, nonce: &[u8], algorithm: Algorithm) -> Self {
		if nonce.len() != algorithm.nonce_len(Mode::Stream) {
			// error here
		}

		match algorithm {
			Algorithm::XChaCha20Poly1305 => {
				let cipher = XChaCha20Poly1305::new_from_slice(key.expose_secret()).unwrap();
				drop(key);

				let stream = EncryptorLE31::from_aead(cipher, nonce.into());
				StreamEncryption::XChaCha20Poly1305(Box::new(stream))
			}
			Algorithm::Aes256Gcm => {
				let cipher = Aes256Gcm::new_from_slice(key.expose_secret()).unwrap();
				drop(key);

				let stream = EncryptorLE31::from_aead(cipher, nonce.into());
				StreamEncryption::Aes256Gcm(Box::new(stream))
			}
		}
	}

	// This should be used for every block, except the final block
	pub fn encrypt_next<'msg, 'aad>(
		&mut self,
		payload: impl Into<Payload<'msg, 'aad>>,
	) -> aead::Result<Vec<u8>> {
		match self {
			StreamEncryption::XChaCha20Poly1305(s) => s.encrypt_next(payload),
			StreamEncryption::Aes256Gcm(s) => s.encrypt_next(payload),
		}
	}

	// This should be used to encrypt the final block of data
	pub fn encrypt_last<'msg, 'aad>(
		self,
		payload: impl Into<Payload<'msg, 'aad>>,
	) -> aead::Result<Vec<u8>> {
		match self {
			StreamEncryption::XChaCha20Poly1305(s) => s.encrypt_last(payload),
			StreamEncryption::Aes256Gcm(s) => s.encrypt_last(payload),
		}
	}
}
