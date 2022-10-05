use std::io::{Read, Seek, Write};

use aead::{
	stream::{DecryptorLE31, EncryptorLE31},
	KeyInit, Payload,
};
use aes_gcm::Aes256Gcm;
use chacha20poly1305::XChaCha20Poly1305;
use secrecy::{ExposeSecret, Secret};
use zeroize::Zeroize;

use crate::{
	error::Error,
	primitives::{Algorithm, Mode, BLOCK_SIZE},
};

pub enum StreamEncryption {
	XChaCha20Poly1305(Box<EncryptorLE31<XChaCha20Poly1305>>),
	Aes256Gcm(Box<EncryptorLE31<Aes256Gcm>>),
}

pub enum StreamDecryption {
	Aes256Gcm(Box<DecryptorLE31<Aes256Gcm>>),
	XChaCha20Poly1305(Box<DecryptorLE31<XChaCha20Poly1305>>),
}

impl StreamEncryption {
	pub fn init(key: Secret<[u8; 32]>, nonce: &[u8], algorithm: Algorithm) -> Result<Self, Error> {
		if nonce.len() != algorithm.nonce_len(Mode::Stream) {
			return Err(Error::NonceLengthMismatch);
		}

		let encryption_object = match algorithm {
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
		};

		Ok(encryption_object)
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
	// This takes ownership of `self` to prevent usage after finalization
	pub fn encrypt_last<'msg, 'aad>(
		self,
		payload: impl Into<Payload<'msg, 'aad>>,
	) -> aead::Result<Vec<u8>> {
		match self {
			StreamEncryption::XChaCha20Poly1305(s) => s.encrypt_last(payload),
			StreamEncryption::Aes256Gcm(s) => s.encrypt_last(payload),
		}
	}

	// This does not handle writing the header
	// I'm unsure whether this should be taking ownership of `reader` and `writer`, but it seems like a good idea
	pub fn encrypt_streams<R, W>(mut self, mut reader: R, mut writer: W) -> Result<(), Error>
	where
		R: Read + Seek,
		W: Write + Seek,
	{
		let mut read_buffer = vec![0u8; BLOCK_SIZE];
		let read_count = reader.read(&mut read_buffer).map_err(Error::Io)?;
		if read_count == BLOCK_SIZE {
			let encrypted_data = self
				.encrypt_next(read_buffer.as_ref())
				.map_err(|_| Error::Encrypt)?;

			// zeroize before writing, so any potential errors won't result in a potential data leak
			read_buffer.zeroize();

			// Using `write` instead of `write_all` so we can check the amount of bytes written
			let write_count = writer.write(&encrypted_data).map_err(Error::Io)?;

			if read_count != write_count + 16 {
				return Err(Error::WriteMismatch);
			}
		} else {
			let encrypted_data = self
				.encrypt_last(read_buffer.as_ref())
				.map_err(|_| Error::Encrypt)?;

			// zeroize before writing, so any potential errors won't result in a potential data leak
			read_buffer.zeroize();

			// Using `write` instead of `write_all` so we can check the amount of bytes written
			let write_count = writer.write(&encrypted_data).map_err(Error::Io)?;

			if read_count != write_count + 16 {
				return Err(Error::WriteMismatch);
			}
		}

		writer.flush().map_err(Error::Io)?;

		Ok(())
	}
}

impl StreamDecryption {
	pub fn init(key: Secret<[u8; 32]>, nonce: &[u8], algorithm: Algorithm) -> Result<Self, Error> {
		if nonce.len() != algorithm.nonce_len(Mode::Stream) {
			return Err(Error::NonceLengthMismatch);
		}

		let decryption_object = match algorithm {
			Algorithm::XChaCha20Poly1305 => {
				let cipher = XChaCha20Poly1305::new_from_slice(key.expose_secret()).unwrap();
				drop(key);

				let stream = DecryptorLE31::from_aead(cipher, nonce.into());
				StreamDecryption::XChaCha20Poly1305(Box::new(stream))
			}
			Algorithm::Aes256Gcm => {
				let cipher = Aes256Gcm::new_from_slice(key.expose_secret()).unwrap();
				drop(key);

				let stream = DecryptorLE31::from_aead(cipher, nonce.into());
				StreamDecryption::Aes256Gcm(Box::new(stream))
			}
		};

		Ok(decryption_object)
	}

	// This should be used for every block, except the final block
	pub fn decrypt_next<'msg, 'aad>(
		&mut self,
		payload: impl Into<Payload<'msg, 'aad>>,
	) -> aead::Result<Vec<u8>> {
		match self {
			StreamDecryption::XChaCha20Poly1305(s) => s.decrypt_next(payload),
			StreamDecryption::Aes256Gcm(s) => s.decrypt_next(payload),
		}
	}

	// This should be used to decrypt the final block of data
	// This takes ownership of `self` to prevent usage after finalization
	pub fn decrypt_last<'msg, 'aad>(
		self,
		payload: impl Into<Payload<'msg, 'aad>>,
	) -> aead::Result<Vec<u8>> {
		match self {
			StreamDecryption::XChaCha20Poly1305(s) => s.decrypt_last(payload),
			StreamDecryption::Aes256Gcm(s) => s.decrypt_last(payload),
		}
	}

	// This does not handle writing the header
	// I'm unsure whether this should be taking ownership of `reader` and `writer`, but it seems like a good idea
	pub fn decrypt_streams<R, W>(mut self, mut reader: R, mut writer: W) -> Result<(), Error>
	where
		R: Read + Seek,
		W: Write + Seek,
	{
		let mut read_buffer = vec![0u8; BLOCK_SIZE];
		let read_count = reader.read(&mut read_buffer).map_err(Error::Io)?;
		if read_count == BLOCK_SIZE {
			let mut decrypted_data = self
				.decrypt_next(read_buffer.as_ref())
				.map_err(|_| Error::Decrypt)?;

			// Using `write` instead of `write_all` so we can check the amount of bytes written
			let write_count = writer.write(&decrypted_data).map_err(Error::Io)?;

			// zeroize before writing, so any potential errors won't result in a potential data leak
			decrypted_data.zeroize();

			if read_count - 16 != write_count {
				return Err(Error::WriteMismatch);
			}
		} else {
			let mut decrypted_data = self
				.decrypt_last(read_buffer[..read_count].as_ref())
				.map_err(|_| Error::Decrypt)?;

			// Using `write` instead of `write_all` so we can check the amount of bytes written
			let write_count = writer.write(&decrypted_data).map_err(Error::Io)?;

			// zeroize before writing, so any potential errors won't result in a potential data leak
			decrypted_data.zeroize();

			if read_count - 16 != write_count {
				return Err(Error::WriteMismatch);
			}
		}

		writer.flush().map_err(Error::Io)?;

		Ok(())
	}
}
