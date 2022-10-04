use std::{
	cell::RefCell,
	io::{Read, Seek, Write},
};

use zeroize::Zeroize;

use crate::{primitives::BLOCK_SIZE, utils::stream::StreamEncryption, error::Error};

// I'm not too sure `RefCell`s are the best choice here
// They provide mutable ownership to the encryptor, and that allows us to have full control over them
pub struct StreamEncryptor<R, W>
where
	R: Read + Seek,
	W: Write + Seek,
{
	stream_object: RefCell<StreamEncryption>,
	reader: RefCell<R>,
	writer: RefCell<W>,
	current_step: i64,
	total_step: i64,
}

pub enum StreamStepType {
	Normal,
	Final,
}

impl<R, W> StreamEncryptor<R, W>
where
	R: Read + Seek,
	W: Write + Seek,
{
	pub fn new(
		stream_object: StreamEncryption,
		source_file: R,
		output_file: W,
		file_size: u32,
	) -> Self {
		let stream_object = RefCell::new(stream_object);
		let reader = RefCell::new(source_file);

		let writer = RefCell::new(output_file);

		let current_step = 0;
		let total_step = (file_size as f32 / BLOCK_SIZE as f32).ceil() as i64;

		Self {
			stream_object,
			reader,
			writer,
			current_step,
			total_step,
		}
	}

	/// This needs to be used in order to determine whether to call `.step()`, or `.finalize()`
	/// If the incorrect function is called, you will receive an error (so make sure this check happens!)
	#[must_use]
	pub fn get_step_type(&self) -> StreamStepType {
		if self.current_step < self.total_step {
			StreamStepType::Normal
		} else {
			StreamStepType::Final
		}
	}

	pub fn step(&mut self) -> Result<(), Error> {
		let mut read_buffer = vec![0u8; BLOCK_SIZE];
		let read_count = self.reader.borrow_mut().read(&mut read_buffer).map_err(Error::Io)?;
		if read_count == BLOCK_SIZE && self.current_step < self.total_step {
			let encrypted_data = self
				.stream_object
				.borrow_mut()
				.encrypt_next(read_buffer.as_ref())
				.map_err(|_| Error::Encrypt)?;

			// zeroize before writing, so any potential errors won't result in a potential data leak
			read_buffer.zeroize();

			// Using `write` instead of `write_all` so we can check the amount of bytes written
			let write_count = self.writer.borrow_mut().write(&encrypted_data).map_err(Error::Io)?;

			if read_count != write_count {
                return Err(Error::WriteMismatch)
			}
		} else {
            return Err(Error::IncorrectStep)
		}

		self.current_step += 1;

        Ok(())
	}

	// Finalize must be called when the `current_step` == `total_step`
	pub fn finalize(self) -> Result<(), Error> {
		let mut read_buffer = vec![0u8; BLOCK_SIZE];
		let read_count = self.reader.borrow_mut().read(&mut read_buffer).map_err(Error::Io)?;

		if read_count != BLOCK_SIZE && self.current_step == self.total_step {
			let encrypted_data = self
				.stream_object
				.into_inner()
				.encrypt_last(read_buffer.as_ref())
				.map_err(|_| Error::Encrypt)?;

			// zeroize before writing, so any potential errors won't result in a potential data leak
			read_buffer.zeroize();

			// Using `write` instead of `write_all` so we can check the amount of bytes written
			let write_count = self.writer.borrow_mut().write(&encrypted_data).map_err(Error::Io)?;

			if read_count != write_count {
                return Err(Error::WriteMismatch)
			}
		} else {
            return Err(Error::IncorrectStep)
		}

        Ok(())
	}
}